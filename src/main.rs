use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

use crate::mode::Mode;
use crate::utils::button::PressType;

mod mode;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("=== LED BOX Starting ===");

    let peripherals = Peripherals::take().unwrap();
    FreeRtos::delay_ms(200);

    // Initialize LED matrix (MAX7219 via SPI)
    let mut spi = utils::led::initialize_spi(
        peripherals.spi2,
        peripherals.pins.gpio8.into(),
        peripherals.pins.gpio9.into(),
        peripherals.pins.gpio10.into(),
    )?;
    utils::led::initialize_matrix_display(&mut spi);
    log::info!("LED matrix initialized");

    // Initialize NVS (clone partition for sharing between subsystems)
    let nvs_partition = EspDefaultNvsPartition::take()?;
    let nvs_for_ble = nvs_partition.clone();

    // Initialize Mode Manager
    let nvs_mode = EspNvs::new(nvs_partition, "TEST", true)?;
    let mut mode_manager = mode::ModeManager::new(nvs_mode)?;
    log::info!("Mode system initialized: {}", mode_manager.current().name());

    // Initialize BLE
    let ble = utils::bluetooth::BluetoothManager::init(nvs_for_ble)?;
    log::info!("BLE initialized");

    // Initialize buttons (red=GPIO3, white=GPIO4)
    let mut buttons = utils::button::Buttons::new(
        peripherals.pins.gpio3.into(),
        peripherals.pins.gpio4.into(),
    )?;
    log::info!("Buttons initialized (red=GPIO3, white=GPIO4)");

    // Show initial mode icon
    display_matrix(&mut spi, &mode_manager.current().icon());

    // Main loop
    loop {
        // Poll buttons
        let red_press = buttons.red.poll();
        let white_press = buttons.white.poll();

        // White long-press: cycle mode (universal, consumed before mode dispatch)
        let white_press = if let Some(PressType::Long) = white_press {
            let next = mode_manager.current().next();
            if let Err(e) = mode_manager.switch_to(next) {
                log::error!("Failed to switch mode: {:?}", e);
            }
            display_matrix(&mut spi, &mode_manager.current().icon());
            FreeRtos::delay_ms(500);
            None // consume the press
        } else {
            white_press
        };

        // Mode-specific behavior
        match mode_manager.current() {
            Mode::Pet => {
                display_matrix(&mut spi, &Mode::Pet.icon());
                if let Some(press) = red_press {
                    log::info!("[Pet] Red: {:?}", press);
                }
                if let Some(PressType::Short) = white_press {
                    log::info!("[Pet] White short press");
                }
            }
            Mode::Pomodoro => {
                display_matrix(&mut spi, &Mode::Pomodoro.icon());
                if let Some(press) = red_press {
                    log::info!("[Pomodoro] Red: {:?}", press);
                }
                if let Some(PressType::Short) = white_press {
                    log::info!("[Pomodoro] White short press");
                }
            }
            Mode::Tools => {
                // Preserve existing BLE → LED display behavior
                let data = ble.get_display_data();
                display_matrix(&mut spi, &data);
                if let Some(press) = red_press {
                    log::info!("[Tools] Red: {:?}", press);
                }
                if let Some(PressType::Short) = white_press {
                    log::info!("[Tools] White short press");
                }
            }
            Mode::Notification => {
                display_matrix(&mut spi, &Mode::Notification.icon());
                if let Some(press) = red_press {
                    log::info!("[Notification] Red: {:?}", press);
                }
                if let Some(PressType::Short) = white_press {
                    log::info!("[Notification] White short press");
                }
            }
            Mode::SmartHome => {
                display_matrix(&mut spi, &Mode::SmartHome.icon());
                if let Some(press) = red_press {
                    log::info!("[SmartHome] Red: {:?}", press);
                }
                if let Some(PressType::Short) = white_press {
                    log::info!("[SmartHome] White short press");
                }
            }
            Mode::Monitor => {
                display_matrix(&mut spi, &Mode::Monitor.icon());
                if let Some(press) = red_press {
                    log::info!("[Monitor] Red: {:?}", press);
                }
                if let Some(PressType::Short) = white_press {
                    log::info!("[Monitor] White short press");
                }
            }
        }

        FreeRtos::delay_ms(50);
    }
}

fn display_matrix<'a>(spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>, data: &[u8]) {
    if data.len() >= 8 {
        for addr in 1..=8u8 {
            spi.write(&[addr, data[(addr - 1) as usize]]).unwrap();
        }
    } else {
        let default = [0x00, 0x66, 0x66, 0x00, 0x00, 0x42, 0x3C, 0x00];
        for (i, &byte) in default.iter().enumerate() {
            spi.write(&[(i + 1) as u8, byte]).unwrap();
        }
    }
}
