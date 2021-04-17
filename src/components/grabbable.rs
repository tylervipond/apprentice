use serde::{Deserialize, Serialize};
use specs::{Component, NullStorage};

#[derive(Component, Deserialize, Serialize, Debug, Clone, Default)]
#[storage(NullStorage)]
pub struct Grabbable {}
