use specs::{Component, NullStorage};

#[derive(Component, Clone, Debug, Default)]
#[storage(NullStorage)]
pub struct WantsToReleaseGrabbed {}
