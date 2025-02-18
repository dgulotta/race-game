use std::time::Duration;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Playback {
    Paused,
    Playing(Duration),
    Fast(Duration),
}
