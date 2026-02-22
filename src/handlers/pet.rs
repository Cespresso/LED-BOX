use crate::assets;
use crate::utils::animation::{AnimationClip, AnimationPlayer};
use crate::utils::button::PressType;

use super::ModeHandler;

static IDLE_FRAMES: &[[u8; 8]] = &[
    assets::FACE_SMILE,
    assets::FACE_SMILE,
    assets::FACE_BLINK,
    assets::FACE_SMILE,
];

static HAPPY_FRAMES: &[[u8; 8]] = &[
    assets::FACE_HAPPY,
    assets::FACE_SMILE,
    assets::FACE_HAPPY,
    assets::FACE_SMILE,
];

static ANGRY_FRAMES: &[[u8; 8]] = &[
    assets::FACE_ANGRY,
    assets::FACE_SAD,
    assets::FACE_ANGRY,
    assets::FACE_SAD,
];

pub struct PetHandler {
    animator: AnimationPlayer,
}

impl PetHandler {
    pub fn new() -> Self {
        let clip = AnimationClip::looping(IDLE_FRAMES, 500);
        Self {
            animator: AnimationPlayer::new(clip),
        }
    }
}

impl ModeHandler for PetHandler {
    fn on_enter(&mut self) -> [u8; 8] {
        *self.animator.current_frame()
    }

    fn on_red_button(&mut self, press: PressType) {
        if press == PressType::Short {
            let clip = AnimationClip::one_shot(HAPPY_FRAMES, 300);
            self.animator.play_interrupt(clip);
        }
    }

    fn on_white_button(&mut self, press: PressType) {
        if press == PressType::Short {
            let clip = AnimationClip::one_shot(ANGRY_FRAMES, 300);
            self.animator.play_interrupt(clip);
        }
    }

    fn tick(&mut self) -> Option<[u8; 8]> {
        self.animator.tick().copied()
    }
}
