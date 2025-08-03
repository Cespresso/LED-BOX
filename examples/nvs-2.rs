use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use esp_idf_svc::sys::nvs_get_str;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let nvs = EspDefaultNvsPartition::take()?;
    let mut nvs_hander = EspNvs::new(nvs, "TEST", true)?;
    // nvs_hander.set_str("key","1").unwrap();
    // nvs_hander.set_i8("KEY_NUM", 9).unwrap();

    // let mut buf = [0u8,128];
    // let value = nvs_hander.get_str("key",&mut buf)?.unwrap();
    let value = nvs_hander.get_i8("KEY_NUM").unwrap().unwrap();

    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("{}" ,value);
    Ok(())
}
