// Data structures for LED BOX functionality - duplicate with -2 suffix

// Original data structure
pub struct LEDMatrix {
    pub data: [u8; 8],
    pub brightness: u8,
    pub enabled: bool,
}

// Duplicate data structure with -2 suffix
pub struct LEDMatrix2 {
    pub data: [u8; 8],
    pub brightness: u8,
    pub enabled: bool,
}

impl LEDMatrix {
    pub fn new() -> Self {
        LEDMatrix {
            data: [0; 8],
            brightness: 0x0F,
            enabled: true,
        }
    }
    
    pub fn set_pattern(&mut self, pattern: [u8; 8]) {
        self.data = pattern;
    }
}

impl LEDMatrix2 {
    pub fn new() -> Self {
        LEDMatrix2 {
            data: [0xFF; 8], // Different default pattern for -2
            brightness: 0x0F,
            enabled: true,
        }
    }
    
    pub fn set_pattern_2(&mut self, pattern: [u8; 8]) {
        self.data = pattern;
    }
}

// Original BLE configuration
pub struct BLEConfig {
    pub service_uuid: &'static str,
    pub characteristic_uuid: &'static str,
    pub device_name: &'static str,
}

// Duplicate BLE configuration with -2 suffix
pub struct BLEConfig2 {
    pub service_uuid: &'static str,
    pub characteristic_uuid: &'static str,
    pub device_name: &'static str,
}

impl BLEConfig {
    pub fn default() -> Self {
        BLEConfig {
            service_uuid: "455aa9f0-2999-43de-81b4-54e0de255927",
            characteristic_uuid: "681285a6-247f-48c6-80ad-68c3dce18585",
            device_name: "LED BOX",
        }
    }
}

impl BLEConfig2 {
    pub fn default() -> Self {
        BLEConfig2 {
            service_uuid: "455aa9f0-2999-43de-81b4-54e0de255928",
            characteristic_uuid: "681285a6-247f-48c6-80ad-68c3dce18586",
            device_name: "LED BOX-2",
        }
    }
}