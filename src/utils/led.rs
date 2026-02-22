use embedded_hal::spi::{Mode, Phase, Polarity};
use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::spi::config::Config;
use esp_idf_hal::spi::{SpiDeviceDriver, SpiDriver, SpiDriverConfig, SPI2};
use esp_idf_hal::sys::EspError;

pub fn initialize_spi<'d>(
    spi: impl Peripheral<P = SPI2> + 'd,
    sclk: AnyOutputPin,
    cs: AnyOutputPin,
    mosi: AnyOutputPin,
) -> Result<SpiDeviceDriver<'d, SpiDriver<'d>>, EspError> {
    let spi_drv = SpiDriver::new(
        spi,
        sclk,
        mosi,
        None::<AnyIOPin>,
        &SpiDriverConfig::new(),
    )?;

    let config = Config::new().baudrate(2.MHz().into()).data_mode(Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    });
    SpiDeviceDriver::new(spi_drv, Some(cs), &config)
}

pub fn initialize_matrix_display<'d>(spi: &mut SpiDeviceDriver<'d, SpiDriver<'d>>) {
    // Power Up Device
    spi.write(&[0x0C, 0x01]).unwrap();
    // Set up Decode Mode (No Decode)
    spi.write(&[0x09, 0x00]).unwrap();
    // Configure Scan Limit (All digits)
    spi.write(&[0x0B, 0x07]).unwrap();
    // Configure Intensity (Maximum)
    spi.write(&[0x0A, 0x0F]).unwrap();
}
