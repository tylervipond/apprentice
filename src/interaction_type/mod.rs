mod copy;
use specs::Entity;
#[derive(Copy, Clone, PartialEq)]
pub enum InteractionType {
    Douse(Entity),
    Light(Entity),
    HideIn(Entity),
    Attack(Entity),
    Grab(Entity),
    Release,
    Disarm(Entity),
    Arm(Entity),
    // Use,
    GoUp(usize),
    GoDown(usize),
    Move(usize),
    Exit(usize),
    Pickup(Entity),
    OpenDoor(Entity),
    CloseDoor(Entity),
    OpenContainer(Entity),
}

impl InteractionType {
    pub fn short_text(&self) -> &str {
        match self {
            Self::Disarm(_) => copy::SHORT_TEXT_DISARM,
            Self::Arm(_) => copy::SHORT_TEXT_ARM,
            Self::Douse(_) => copy::SHORT_TEXT_DOUSE,
            Self::Light(_) => copy::SHORT_TEXT_LIGHT,
            Self::Grab(_) => copy::SHORT_TEXT_GRAB,
            Self::HideIn(_) => copy::SHORT_TEXT_HIDE,
            Self::Attack(_) => copy::SHORT_TEXT_ATTACK,
            Self::Pickup(_) => copy::SHORT_TEXT_PICKUP,
            Self::OpenContainer(_) => copy::SHORT_TEXT_OPEN,
            Self::OpenDoor(_) => copy::SHORT_TEXT_OPEN_DOOR,
            Self::CloseDoor(_) => copy::SHORT_TEXT_CLOSE_DOOR,
            Self::GoDown(_) => copy::SHORT_TEXT_GO_DOWN,
            Self::GoUp(_) => copy::SHORT_TEXT_GO_UP,
            Self::Exit(_) => copy::SHORT_TEXT_EXIT,
            Self::Move(_) => copy::SHORT_TEXT_MOVE,
            Self::Release => copy::SHORT_TEXT_RELEASE
        }
    }
    pub fn descriptive_text(&self) -> &str {
        match self {
            Self::Disarm(_) => copy::DESCRIPTION_DISARM,
            Self::Arm(_) => copy::DESCRIPTION_ARM,
            Self::Douse(_) => copy::DESCRIPTION_DOUSE,
            Self::Light(_) => copy::DESCRIPTION_LIGHT,
            Self::Grab(_) => copy::DESCRIPTION_GRAB,
            Self::HideIn(_) => copy::DESCRIPTION_HIDE,
            Self::Attack(_) => copy::DESCRIPTION_ATTACK,
            Self::Pickup(_) => copy::DESCRIPTION_PICKUP,
            Self::OpenContainer(_) => copy::DESCRIPTION_OPEN,
            Self::OpenDoor(_) => copy::DESCRIPTION_OPEN_DOOR,
            Self::CloseDoor(_) => copy::DESCRIPTION_CLOSE_DOOR,
            Self::GoDown(_) => copy::DESCRIPTION_GO_DOWN,
            Self::GoUp(_) => copy::DESCRIPTION_GO_UP,
            Self::Exit(_) => copy::DESCRIPTION_EXIT,
            Self::Move(_) => copy::DESCRIPTION_MOVE,
            Self::Release => copy::DESCRIPTION_RELEASE
        }
    }
}
