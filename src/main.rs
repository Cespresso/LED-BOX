use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver};
use esp_idf_svc::nvs::EspDefaultNvsPartition;

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

    // Initialize BLE
    let nvs_partition = EspDefaultNvsPartition::take()?;
    let ble = utils::bluetooth::BluetoothManager::init(nvs_partition)?;
    log::info!("BLE initialized");

    // Initialize buttons (red=GPIO3, white=GPIO4)
    let mut buttons = utils::button::Buttons::new(
        peripherals.pins.gpio3.into(),
        peripherals.pins.gpio4.into(),
    )?;
    log::info!("Buttons initialized (red=GPIO3, white=GPIO4)");

    // Main loop
    loop {
        // Poll buttons
        if let Some(press) = buttons.red.poll() {
            log::info!("Red button: {:?}", press);
        }
        if let Some(press) = buttons.white.poll() {
            log::info!("White button: {:?}", press);
        }

        // Update LED display from BLE data
        let data = ble.get_display_data();
        display_matrix(&mut spi, &data);

        FreeRtos::delay_ms(50);
    }
}

fn display_matrix<'a>(spi: &mut SpiDeviceDriver<'a, SpiDriver<'a>>, data: &[u8]) {
    if data.len() >= 8 {
        for addr in 1..=8u8 {
            spi.write(&[addr, data[(addr - 1) as usize]]).unwrap();
        }
    } else {
        // Default smiley face
        let default = [0x00, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x00];
        for (i, &byte) in default.iter().enumerate() {
            spi.write(&[(i + 1) as u8, byte]).unwrap();
        }
    }
}
