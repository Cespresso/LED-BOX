pub mod default;
pub mod pet;
pub mod tools;

use crate::mode::Mode;
use crate::utils::button::PressType;

/// Common interface for mode-specific behavior.
/// Each mode implements this trait to handle buttons, BLE data, and display updates.
pub trait ModeHandler {
    /// Called when entering this mode. Returns the initial frame to display.
    fn on_enter(&mut self) -> [u8; 8];

    /// Called on red button press.
    fn on_red_button(&mut self, _press: PressType) {}

    /// Called on white button short press.
    fn on_white_button(&mut self, _press: PressType) {}

    /// Called when BLE display data is received.
    fn on_ble_data(&mut self, _data: [u8; 8]) {}

    /// Called every tick (~50ms). Returns Some(frame) if display should update.
    fn tick(&mut self) -> Option<[u8; 8]>;
}

/// Create the appropriate handler for the given mode.
pub fn create_handler(mode: Mode, ble_display_data: [u8; 8]) -> Box<dyn ModeHandler> {
    match mode {
        Mode::Pet => Box::new(pet::PetHandler::new()),
        Mode::Tools => Box::new(tools::ToolsHandler::new(ble_display_data)),
        _ => Box::new(default::DefaultHandler::new(mode)),
    }
}
