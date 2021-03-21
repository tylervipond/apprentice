use specs::{Component, DenseVecStorage};

#[derive(Component, Debug, Clone)]
pub struct WantsToGoDownStairs {
    pub idx: usize,
}
