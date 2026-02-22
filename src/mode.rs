use esp_idf_svc::nvs::{EspNvs, NvsDefault};
use esp_idf_hal::sys::EspError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    Pet = 0,
    Pomodoro = 1,
    Tools = 2,
    Notification = 3,
    SmartHome = 4,
    Monitor = 5,
}

const MODE_COUNT: u8 = 6;

impl Mode {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Mode::Pet,
            1 => Mode::Pomodoro,
            2 => Mode::Tools,
            3 => Mode::Notification,
            4 => Mode::SmartHome,
            5 => Mode::Monitor,
            _ => Mode::Pet,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Mode::Pet => "Pet",
            Mode::Pomodoro => "Pomodoro",
            Mode::Tools => "Tools",
            Mode::Notification => "Notification",
            Mode::SmartHome => "SmartHome",
            Mode::Monitor => "Monitor",
        }
    }

    pub fn next(self) -> Self {
        Mode::from_u8((self as u8 + 1) % MODE_COUNT)
    }

    /// Stub 8x8 icon for each mode (placeholder until Phase 0-6 assets)
    pub fn icon(&self) -> [u8; 8] {
        match self {
            Mode::Pet =>          [0x00, 0x66, 0x66, 0x00, 0x00, 0x42, 0x3C, 0x00],
            Mode::Pomodoro =>     [0x3C, 0x42, 0x42, 0x3C, 0x18, 0x18, 0x3C, 0x00],
            Mode::Tools =>        [0x18, 0x18, 0x7E, 0x7E, 0x18, 0x18, 0x18, 0x00],
            Mode::Notification => [0x18, 0x3C, 0x3C, 0x3C, 0x7E, 0x00, 0x18, 0x00],
            Mode::SmartHome =>    [0x18, 0x3C, 0x7E, 0x7E, 0x42, 0x42, 0xFF, 0x00],
            Mode::Monitor =>      [0x7E, 0x42, 0x42, 0x42, 0x7E, 0x18, 0x7E, 0x00],
        }
    }
}

pub struct ModeManager {
    current_mode: Mode,
    nvs: EspNvs<NvsDefault>,
}

impl ModeManager {
    pub fn new(nvs: EspNvs<NvsDefault>) -> Result<Self, EspError> {
        let current_mode = match nvs.get_u8("MODE")? {
            Some(v) => {
                let mode = Mode::from_u8(v);
                log::info!("Loaded mode from NVS: {}", mode.name());
                mode
            }
            None => {
                log::info!("No saved mode, defaulting to Pet");
                Mode::Pet
            }
        };
        Ok(Self { current_mode, nvs })
    }

    pub fn current(&self) -> Mode {
        self.current_mode
    }

    pub fn switch_to(&mut self, new_mode: Mode) -> Result<(), EspError> {
        if self.current_mode == new_mode {
            return Ok(());
        }
        log::info!(
            "Mode: {} -> {}",
            self.current_mode.name(),
            new_mode.name()
        );
        self.current_mode = new_mode;
        self.nvs.set_u8("MODE", new_mode as u8)?;
        Ok(())
    }
}
