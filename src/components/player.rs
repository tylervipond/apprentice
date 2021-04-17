use serde::{Deserialize, Serialize};
use specs::{Component, NullStorage};

#[derive(Component, Serialize, Deserialize, Debug, Clone, Default)]
#[storage(NullStorage)]
pub struct Player {}
