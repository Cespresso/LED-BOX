use std::time::Instant;

/// How the animation behaves when it reaches the last frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackMode {
    /// Loop forever, wrapping back to frame 0 after the last frame.
    Loop,
    /// Play once, then hold on the last frame.
    OneShot,
}

/// Immutable definition of an animation sequence.
#[derive(Debug, Clone)]
pub struct AnimationClip {
    frames: Vec<[u8; 8]>,
    frame_duration_ms: u32,
    playback_mode: PlaybackMode,
}

impl AnimationClip {
    pub fn new(frames: Vec<[u8; 8]>, frame_duration_ms: u32, playback_mode: PlaybackMode) -> Self {
        Self {
            frames,
            frame_duration_ms,
            playback_mode,
        }
    }

    pub fn looping(frames: Vec<[u8; 8]>, frame_duration_ms: u32) -> Self {
        Self::new(frames, frame_duration_ms, PlaybackMode::Loop)
    }

    pub fn one_shot(frames: Vec<[u8; 8]>, frame_duration_ms: u32) -> Self {
        Self::new(frames, frame_duration_ms, PlaybackMode::OneShot)
    }
}

/// Stateful player that tracks the current frame and elapsed time.
pub struct AnimationPlayer {
    clip: AnimationClip,
    current_frame: usize,
    last_tick: Instant,
    finished: bool,
}

impl AnimationPlayer {
    /// Create a new player and start playback immediately.
    pub fn new(clip: AnimationClip) -> Self {
        Self {
            clip,
            current_frame: 0,
            last_tick: Instant::now(),
            finished: false,
        }
    }

    /// Advance the animation based on elapsed time.
    /// Returns `Some(&[u8; 8])` if the frame changed, `None` otherwise.
    pub fn tick(&mut self) -> Option<&[u8; 8]> {
        if self.clip.frames.is_empty() || self.finished {
            return None;
        }

        let elapsed = self.last_tick.elapsed().as_millis() as u32;
        if elapsed < self.clip.frame_duration_ms {
            return None;
        }

        self.last_tick = Instant::now();
        let next = self.current_frame + 1;

        if next >= self.clip.frames.len() {
            match self.clip.playback_mode {
                PlaybackMode::Loop => self.current_frame = 0,
                PlaybackMode::OneShot => {
                    self.finished = true;
                    return None;
                }
            }
        } else {
            self.current_frame = next;
        }

        Some(&self.clip.frames[self.current_frame])
    }

    /// Get the current frame data without advancing.
    pub fn current_frame(&self) -> &[u8; 8] {
        &self.clip.frames[self.current_frame]
    }

    /// Whether a OneShot animation has completed.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Reset the animation to the beginning.
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_tick = Instant::now();
        self.finished = false;
    }

    /// Replace the current clip and reset playback.
    pub fn set_clip(&mut self, clip: AnimationClip) {
        self.clip = clip;
        self.reset();
    }
}
