use super::ModeHandler;

pub struct ToolsHandler {
    display_data: [u8; 8],
    dirty: bool,
}

impl ToolsHandler {
    pub fn new(initial_data: [u8; 8]) -> Self {
        Self {
            display_data: initial_data,
            dirty: false,
        }
    }
}

impl ModeHandler for ToolsHandler {
    fn on_enter(&mut self) -> [u8; 8] {
        self.display_data
    }

    fn on_ble_data(&mut self, data: [u8; 8]) {
        self.display_data = data;
        self.dirty = true;
    }

    fn tick(&mut self) -> Option<[u8; 8]> {
        if self.dirty {
            self.dirty = false;
            Some(self.display_data)
        } else {
            None
        }
    }
}
