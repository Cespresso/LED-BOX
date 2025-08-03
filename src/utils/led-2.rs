use embedded_hal::spi::{Mode, Phase, Polarity};
use esp_idf_hal::gpio;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::spi::config::Config;
use esp_idf_hal::spi::SpiDriverConfig;
use esp_idf_svc::hal::spi::{SpiDeviceDriver, SpiDriver};

pub fn initialize_spi_2<'d>(peripherals:Peripherals) -> SpiDeviceDriver<'d, SpiDriver<'d>>{
    // Use different GPIO pins for the second LED matrix
    let sclk = peripherals.pins.gpio18;
    let cs = peripherals.pins.gpio19;
    let mosi = peripherals.pins.gpio21;

    // Instantiate SPI Driver for second matrix
    let spi_drv = SpiDriver::new(
        peripherals.spi3,
        sclk,
        mosi,
        None::<gpio::AnyIOPin>,
        &SpiDriverConfig::new(),
    )
        .unwrap();

    // Configure Parameters for SPI device
    let config = Config::new().baudrate(2.MHz().into()).data_mode(Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    });
    return SpiDeviceDriver::new(spi_drv, Some(cs), &config).unwrap()
}
pub fn initialize_matrix_display_2<'d>(spi: &mut SpiDeviceDriver<'d, SpiDriver<'d>>){

    // 1.a) Power Up Device

    // - Prepare Data to be Sent
    // 8-bit Data/Command Corresponding to Matrix Power Up
    let data: u8 = 0x01;
    // 4-bit Address of Shutdown Mode Command
    let addr: u8 = 0x0C;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    // Note that write method handles the CS pin state
    spi.write(&send_array).unwrap();

    // 1.b) Set up Decode Mode

    // - Prepare Information to be Sent
    // 8-bit Data/Command Corresponding to No Decode Mode
    let data: u8 = 0x00;
    // 4-bit Address of Decode Mode Command
    let addr: u8 = 0x09;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    spi.write(&send_array).unwrap();

    // 1.c) Configure Scan Limit

    // - Prepare Information to be Sent
    // 8-bit Data/Command Corresponding to Scan Limit Displaying all digits
    let data: u8 = 0x07;
    // 4-bit Address of Scan Limit Command
    let addr: u8 = 0x0B;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    spi.write(&send_array).unwrap();

    // // 1.c) Configure Intensity
    //
    // // - Prepare Information to be Sent
    // // 8-bit Data/Command Corresponding to (15/32 Duty Cycle) Medium Intensity
    let data: u8 = 0x0f;
    // 4-bit Address of Intensity Control Command
    let addr: u8 = 0x0A;
    // Package into array to pass to SPI write method
    // Write method will grab array and send all data in it
    let send_array: [u8; 2] = [addr, data];

    // - Send Data
    // Shift in 16 bits by passing send_array (bits will be shifted MSB first)
    spi.write(&send_array).unwrap()
}