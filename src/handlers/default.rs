use crate::mode::Mode;

use super::ModeHandler;

/// Stub handler for modes not yet implemented.
/// Displays the mode's icon statically.
pub struct DefaultHandler {
    icon: [u8; 8],
}

impl DefaultHandler {
    pub fn new(mode: Mode) -> Self {
        Self { icon: mode.icon() }
    }
}

impl ModeHandler for DefaultHandler {
    fn on_enter(&mut self) -> [u8; 8] {
        self.icon
    }

    fn tick(&mut self) -> Option<[u8; 8]> {
        None
    }
}
