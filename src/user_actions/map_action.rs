use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Debug)]
pub enum MapAction {
    #[cfg(debug_assertions)]
    ShowDebugMenu,
    ActLeft,
    ActRight,
    ActUp,
    ActDown,
    ActUpLeft,
    ActUpRight,
    ActDownLeft,
    ActDownRight,
    AutoActLeft,
    AutoActRight,
    AutoActUp,
    AutoActDown,
    AutoActUpLeft,
    AutoActUpRight,
    AutoActDownLeft,
    AutoActDownRight,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveUpLeft,
    MoveUpRight,
    MoveDownLeft,
    MoveDownRight,
    SearchContainer,
    StayStill,
    OpenDoor,
    PickupItem,
    ShowInventoryMenu,
    ShowDropMenu,
    ShowActionMenu,
    ShowEquipmentMenu,
    SearchHidden,
    DisarmTrap,
    ArmTrap,
    GrabFurniture,
    ReleaseFurniture,
    Attack,
    Hide,
    Exit,
    LeaveDungeon,
    Interact,
    ScrollLeft,
    ScrollRight,
    ScrollDown,
    ScrollUp,
}

impl MapAction {
    pub fn actions() -> Box<[Self]> {
        Box::new([
            Self::ActLeft,
            Self::ActRight,
            Self::ActUp,
            Self::ActDown,
            Self::ActUpLeft,
            Self::ActUpRight,
            Self::ActDownLeft,
            Self::ActDownRight,
            Self::AutoActLeft,
            Self::AutoActRight,
            Self::AutoActUp,
            Self::AutoActDown,
            Self::AutoActUpLeft,
            Self::AutoActUpRight,
            Self::AutoActDownLeft,
            Self::AutoActDownRight,
            Self::MoveLeft,
            Self::MoveRight,
            Self::MoveUp,
            Self::MoveDown,
            Self::MoveUpLeft,
            Self::MoveUpRight,
            Self::MoveDownLeft,
            Self::MoveDownRight,
            Self::SearchContainer,
            Self::StayStill,
            Self::OpenDoor,
            Self::PickupItem,
            Self::ShowInventoryMenu,
            Self::ShowDropMenu,
            Self::ShowActionMenu,
            Self::ShowEquipmentMenu,
            Self::SearchHidden,
            Self::DisarmTrap,
            Self::ArmTrap,
            Self::GrabFurniture,
            Self::ReleaseFurniture,
            Self::Attack,
            Self::Hide,
            Self::Exit,
            Self::LeaveDungeon,
            Self::Interact,
            Self::ScrollLeft,
            Self::ScrollRight,
            Self::ScrollDown,
            Self::ScrollUp,
        ])
    }
}

impl Display for MapAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            String::from(match self {
                Self::ActLeft => "Act Left Menu",
                Self::ActRight => "Act Right Menu",
                Self::ActUp => "Act Up Menu",
                Self::ActDown => "Act Down Menu",
                Self::ActUpLeft => "Act Up-Left Menu",
                Self::ActUpRight => "Act Up-Right Menu",
                Self::ActDownLeft => "Act Down-Left Menu",
                Self::ActDownRight => "Act Down-Right Menu",
                Self::AutoActLeft => "Move/Act Left",
                Self::AutoActRight => "Move/Act Right",
                Self::AutoActUp => "Move/Act Up",
                Self::AutoActDown => "Move/Act Down",
                Self::AutoActUpLeft => "Move/Act Up Left",
                Self::AutoActUpRight => "Move/Act Up Right",
                Self::AutoActDownLeft => "Move/Act Down Left",
                Self::AutoActDownRight => "Move/Act Down Right",
                #[cfg(debug_assertions)]
                Self::ShowDebugMenu => "Show Debug Menu",
                Self::MoveLeft => "Move Left",
                Self::MoveRight => "Move Right",
                Self::MoveUp => "Move Up",
                Self::MoveDown => "Move Down",
                Self::MoveUpLeft => "Move Up Left",
                Self::MoveUpRight => "Move Up Right",
                Self::MoveDownLeft => "Move Down Left",
                Self::MoveDownRight => "Move Down Right",
                Self::SearchContainer => "Search Container",
                Self::StayStill => "Stay Still",
                Self::OpenDoor => "Open Door",
                Self::PickupItem => "Pickup Item",
                Self::ShowInventoryMenu => "Show Inventory Menu",
                Self::ShowDropMenu => "Show Drop Menu",
                Self::ShowActionMenu => "Show Action Menu",
                Self::ShowEquipmentMenu => "Show Equipment Menu",
                Self::SearchHidden => "Search Area",
                Self::DisarmTrap => "Disarm Trap",
                Self::ArmTrap => "Arm Trap",
                Self::GrabFurniture => "Grab Furniture",
                Self::ReleaseFurniture => "Release Furniture",
                Self::Attack => "Attack",
                Self::Hide => "Hide",
                Self::Exit => "Exit",
                Self::LeaveDungeon => "Leave Dungeon",
                Self::Interact => "Interact",
                Self::ScrollLeft => "Scroll Left",
                Self::ScrollRight => "Scroll Right",
                Self::ScrollDown => "Scroll Down",
                Self::ScrollUp => "Scroll Up",
            })
        )
    }
}
