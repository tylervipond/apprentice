#[cfg(not(target_arch = "wasm32"))]
mod not_wasm;
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(not(target_arch = "wasm32"))]
pub use not_wasm::Music;
#[cfg(target_arch = "wasm32")]
pub use wasm::Music;

