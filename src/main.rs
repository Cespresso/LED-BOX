use embedded_hal::spi::*;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::config::Config;
use esp_idf_hal::spi::*;

mod utils;
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Start LED BOX");
    // Setup handler for device peripherals
    let peripherals = Peripherals::take().unwrap();
    // Create handles for SPI pins
    let mut spi = utils::led::initialize_spi(peripherals);
    utils::led::initialize_matrix_display(&mut spi);

    let data: u8 = 0x0f;
    loop {
        FreeRtos::delay_ms(3000_u32);
        for addr in 1..9 {
            let send_array: [u8; 2] = [addr, data];
            spi.write(&send_array).unwrap();
        }
        spi.write(&[1, 0x81]).unwrap();
        spi.write(&[2, 0x42]).unwrap();
        spi.write(&[3, 0x24]).unwrap();
        spi.write(&[4, 0x18]).unwrap();
        spi.write(&[5, 0x18]).unwrap();
        spi.write(&[6, 0x24]).unwrap();
        spi.write(&[7, 0x42]).unwrap();
        spi.write(&[8, 0x81]).unwrap();
        FreeRtos::delay_ms(3000_u32);
        for addr in 1..9 {
            let send_array: [u8; 2] = [addr, data];
            spi.write(&send_array).unwrap();
        }
        spi.write(&[1, 0x7E]).unwrap();
        spi.write(&[2, 0x81]).unwrap();
        spi.write(&[3, 0x81]).unwrap();
        spi.write(&[4, 0x81]).unwrap();
        spi.write(&[5, 0x81]).unwrap();
        spi.write(&[6, 0x81]).unwrap();
        spi.write(&[7, 0x81]).unwrap();
        spi.write(&[8, 0x7E]).unwrap();
    }
}
