use crate::{
    direction::Direction,
    tile::TileType,
    ui::{input::key_name, settings::Settings},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize, Hash)]
pub enum Action {
    RotCW,
    RotCCW,
    Flip,
    ToggleLights,
    Reverse,
    SelectModify,
    SelectErase,
    SelectPath,
    SelectTile(TileType),
    Scroll(Direction),
    Undo,
    Redo,
    Delete,
    Home,
    Settings,
    Keys,
    Copy,
    Edit,
    Start,
    StepBack,
    Pause,
    StepForward,
    Play,
    FastForward,
    End,
    Seek(usize),
}

impl Action {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::RotCW => "Rotate clockwise",
            Self::RotCCW => "Rotate counterclockwise",
            Self::Flip => "Flip",
            Self::ToggleLights => "Toggle lights",
            Self::Reverse => "Reverse",
            Self::SelectModify => "Select/Move",
            Self::SelectPath => "Draw paths",
            Self::SelectErase => "Clear",
            Self::SelectTile(t) => t.name(),
            Self::Scroll(d) => d.name(),
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Delete => "Delete",
            Self::Home => "Select level",
            Self::Settings => "Settings",
            Self::Keys => "Show/hide keyboard commands",
            Self::Copy => "Copy another level",
            Self::Edit => "Edit course",
            Self::Start => "Seek to start",
            Self::StepBack => "Step backward",
            Self::Pause => "Pause",
            Self::StepForward => "Step forward",
            Self::Play => "Play",
            Self::FastForward => "Fast forward",
            Self::End => "Seek to end",
            Self::Seek(_) => "Seek",
        }
    }

    pub fn name_with_key_hint(&self, settings: &Settings) -> String {
        match settings.keys.get(self) {
            Some(code) => {
                format!("{} ({})", self.name(), key_name(*code))
            }
            _ => self.name().to_string(),
        }
    }

    pub const fn is_active_when_racing(&self) -> bool {
        matches!(
            self,
            Self::Scroll(_)
                | Self::Start
                | Self::StepBack
                | Self::Pause
                | Self::StepForward
                | Self::Play
                | Self::FastForward
                | Self::End
        )
    }

    pub const fn is_active_when_editing(&self) -> bool {
        !matches!(self, Self::Start | Self::StepBack | Self::Pause | Self::End)
    }

    pub const fn can_start_sim(&self) -> bool {
        matches!(self, Self::StepForward | Self::Play | Self::FastForward)
    }
}
