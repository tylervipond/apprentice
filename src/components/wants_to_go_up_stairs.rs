use specs::{Component, DenseVecStorage};

#[derive(Component, Debug, Clone)]
pub struct WantsToGoUpStairs {
    pub idx: usize,
}
