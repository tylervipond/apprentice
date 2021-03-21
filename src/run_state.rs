use crate::{components::equipable::EquipmentPositions, user_actions::MapAction};
use specs::Entity;


#[derive(PartialEq, Clone, Copy)]
pub enum TargetIntent {
    OpenContainer,
    Grab,
    Disarm,
    Arm,
    HideIn,
    Pickup,
    OpenDoor,
    Attack,
    Light,
    Douse
}

#[derive(PartialEq, Clone)]
pub enum RunState {
    AwaitingInput {
        offset_x: i32,
        offset_y: i32,
    },
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ItemUseTargeting {
        range: u32,
        item: Entity,
    },
    InteractAtIdx {
        idx: usize,
        target_menu_idx: usize,
        interaction_menu_idx: usize,
        interaction_menu_active: bool
    },
    InteractionTypeEntityTargeting {
        target_idx: usize,
        targets: Box<[Entity]>,
        intent: TargetIntent,
        cta: Option<&'static str>,
    },
    InteractiveEntityTargeting {
        target_idx: usize,
    },
    InteractMenu {
        highlighted: usize,
        target: Entity,
    },
    OpenContainerMenu {
        highlighted: usize,
        container: Entity,
    },
    InventoryMenu {
        highlighted: usize,
    },
    DropItemMenu {
        highlighted: usize,
    },
    EquipMenu {
        highlighted: usize,
        position: EquipmentPositions,
    },
    EquipmentMenu {
        highlighted: usize,
        action_highlighted: usize,
        action_menu: bool,
    },
    MainMenu {
        highlighted: usize,
    },
    ExitGameMenu {
        highlighted: usize,
    },
    ActionMenu {
        highlighted: usize,
    },
    #[cfg(debug_assertions)]
    DebugMenu {
        highlighted: usize,
    },
    DeathScreen,
    IntroScreen,
    FailureScreen,
    SuccessScreen,
    OptionsScreen { highlighted: usize },
    SetKey { action: MapAction, highlighted: usize },
    CreditsScreen,
    LoadingScreen {
        count_down: u32,
    },
    SavingScreen {
        count_down: u32,
    },
    NewGameScreen {
        count_down: u32,
    },
}
