use std::time::Instant;

use crate::assets;
use crate::utils::animation::{AnimationClip, AnimationPlayer};
use crate::utils::button::PressType;

use super::ModeHandler;

/// Auto-clear timeout for WaitingInput state (5 minutes).
/// If no new BLE signal arrives within this period, the notification
/// is automatically cleared — prevents infinite blinking when a
/// Claude Code session exits without sending a cleanup signal.
const WAITING_TIMEOUT_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq)]
enum NotificationState {
    /// No active notification — display idle bell icon.
    Idle,
    /// Claude Code is waiting for user input (permission prompt, idle prompt).
    WaitingInput,
    /// Claude Code finished responding.
    TaskComplete,
}

/// Handler for Claude Code notification mode.
///
/// Receives BLE commands from the PC-side hook script:
/// - `data[0] = 0x01` → Waiting for input (bell blink loop)
/// - `data[0] = 0x02` → Task complete (checkmark one-shot)
///
/// Red button short-press clears the current notification.
/// WaitingInput state auto-clears after WAITING_TIMEOUT_MS.
pub struct NotificationHandler {
    animator: AnimationPlayer,
    state: NotificationState,
    /// Timestamp when WaitingInput state was entered (for auto-clear timeout).
    waiting_since: Option<Instant>,
}

impl NotificationHandler {
    pub fn new() -> Self {
        let clip = AnimationClip::looping(&[assets::ICON_NOTIFICATION], 1000);
        Self {
            animator: AnimationPlayer::new(clip),
            state: NotificationState::Idle,
            waiting_since: None,
        }
    }

    fn set_state(&mut self, new_state: NotificationState) {
        if self.state == new_state {
            return;
        }
        self.state = new_state;
        self.waiting_since = match new_state {
            NotificationState::WaitingInput => Some(Instant::now()),
            _ => None,
        };
        match new_state {
            NotificationState::Idle => {
                let clip = AnimationClip::looping(&[assets::ICON_NOTIFICATION], 1000);
                self.animator.set_clip(clip);
            }
            NotificationState::WaitingInput => {
                let clip = AnimationClip::looping(assets::NOTIF_WAITING_FRAMES, 500);
                self.animator.set_clip(clip);
            }
            NotificationState::TaskComplete => {
                let clip = AnimationClip::one_shot(assets::NOTIF_COMPLETE_FRAMES, 800);
                self.animator.set_clip(clip);
            }
        }
    }
}

impl ModeHandler for NotificationHandler {
    fn on_enter(&mut self) -> [u8; 8] {
        assets::ICON_NOTIFICATION
    }

    fn on_red_button(&mut self, press: PressType) {
        if press == PressType::Short {
            log::info!("[Notification] Clear notification");
            self.set_state(NotificationState::Idle);
        }
    }

    fn on_ble_data(&mut self, data: [u8; 8]) {
        match data[0] {
            0x01 => {
                log::info!("[Notification] Waiting for input");
                self.set_state(NotificationState::WaitingInput);
            }
            0x02 => {
                log::info!("[Notification] Task complete");
                self.set_state(NotificationState::TaskComplete);
            }
            other => {
                log::warn!("[Notification] Unknown notification type: 0x{:02X}", other);
            }
        }
    }

    fn tick(&mut self) -> Option<[u8; 8]> {
        // Auto-clear WaitingInput after timeout (session may have ended without cleanup)
        if self.state == NotificationState::WaitingInput {
            if let Some(since) = self.waiting_since {
                if since.elapsed().as_millis() as u64 >= WAITING_TIMEOUT_MS {
                    log::info!("[Notification] WaitingInput timed out, returning to Idle");
                    self.set_state(NotificationState::Idle);
                }
            }
        }
        // When task-complete one-shot finishes, return to idle
        if self.state == NotificationState::TaskComplete && self.animator.is_finished() {
            self.set_state(NotificationState::Idle);
        }
        self.animator.tick().copied()
    }
}
