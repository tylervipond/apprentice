use serde::{Deserialize, Serialize};
use specs::{Component, NullStorage};

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
#[storage(NullStorage)]
pub struct OnFire {}
