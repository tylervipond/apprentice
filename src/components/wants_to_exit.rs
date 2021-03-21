use specs::{Component, DenseVecStorage};

#[derive(Component, Debug, Clone)]
pub struct WantsToExit {
    pub idx: usize,
}
