use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

use crate::mode::Mode;
use crate::utils::bluetooth::{BleCommand, BluetoothManager};
use crate::utils::button::{Buttons, PressType};
use crate::utils::led::Display;

mod assets;
mod handlers;
mod mode;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("=== LED BOX Starting ===");

    let peripherals = Peripherals::take().unwrap();
    FreeRtos::delay_ms(200);

    // Initialize LED matrix (MAX7219 via SPI)
    let mut display = Display::new(
        peripherals.spi2,
        peripherals.pins.gpio8.into(),
        peripherals.pins.gpio9.into(),
        peripherals.pins.gpio10.into(),
    )?;
    log::info!("LED matrix initialized");

    // Initialize NVS and Mode Manager
    let nvs_partition = EspDefaultNvsPartition::take()?;
    let nvs_mode = EspNvs::new(nvs_partition, "TEST", true)?;
    let mut mode_manager = mode::ModeManager::new(nvs_mode)?;
    log::info!("Mode system initialized: {}", mode_manager.current().name());

    // Initialize BLE
    let ble = BluetoothManager::init(mode_manager.current() as u8)?;
    log::info!("BLE initialized");

    // Initialize buttons (red=GPIO3, white=GPIO4)
    let mut buttons = Buttons::new(
        peripherals.pins.gpio3.into(),
        peripherals.pins.gpio4.into(),
    )?;
    log::info!("Buttons initialized (red=GPIO3, white=GPIO4)");

    // Create handler for current mode
    let mut handler = handlers::create_handler(mode_manager.current(), ble.get_display_data());
    display.show(&handler.on_enter());

    // Main loop
    loop {
        let red_press = buttons.red.poll();
        let white_press = buttons.white.poll();

        // Process BLE commands
        if let Some(cmd) = ble.take_command() {
            match cmd {
                BleCommand::SwitchMode(m) => {
                    let new_mode = Mode::from_u8(m);
                    if let Err(e) = mode_manager.switch_to(new_mode) {
                        log::error!("BLE switch_to failed: {:?}", e);
                    }
                    ble.notify_mode_change(mode_manager.current() as u8);
                    handler =
                        handlers::create_handler(mode_manager.current(), ble.get_display_data());
                    display.show(&handler.on_enter());
                }
                BleCommand::SetDisplayData(data) => {
                    handler.on_ble_data(data);
                }
                BleCommand::SetToolsSubMode(submode) => {
                    handler.on_ble_submode(submode);
                }
                BleCommand::SetBrightness(level) => {
                    display.set_intensity(level);
                    ble.notify_brightness_change(level);
                    log::info!("Brightness set to {}", level);
                }
            }
        }

        // White long-press: cycle mode (universal, consumed before mode dispatch)
        let white_press = if let Some(PressType::Long) = white_press {
            let next = mode_manager.current().next();
            if let Err(e) = mode_manager.switch_to(next) {
                log::error!("Failed to switch mode: {:?}", e);
            }
            ble.notify_mode_change(mode_manager.current() as u8);
            handler = handlers::create_handler(mode_manager.current(), ble.get_display_data());
            display.show(&handler.on_enter());
            FreeRtos::delay_ms(500);
            None // consume the press
        } else {
            white_press
        };

        // Mode-specific button handling
        if let Some(press) = red_press {
            log::info!("[{}] Red: {:?}", mode_manager.current().name(), press);
            handler.on_red_button(press);
        }
        if let Some(press) = white_press {
            log::info!("[{}] White: {:?}", mode_manager.current().name(), press);
            handler.on_white_button(press);
        }

        // Display update
        if let Some(frame) = handler.tick() {
            display.show(&frame);
        }

        FreeRtos::delay_ms(50);
    }
}
