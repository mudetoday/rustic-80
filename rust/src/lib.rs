//! Rustic-80 core: a Rust port of the TIC-80 fantasy computer.
//!
//! This crate is the first slice of the port — the 96K RAM map, the draw
//! API, input, the system font, and a sandboxed Lua cart runner. The C
//! studio in the repository root is still the full TIC-80 until each
//! module moves over.

mod draw;
mod font;
mod machine;
mod ram;
mod script;
mod text;

pub use machine::{Input, Machine};
pub use ram::{
    GAMEPADS, HEIGHT, MAP, MAP_HEIGHT, MAP_WIDTH, PALETTE, RAM_SIZE, SPRITES, SWEETIE16, TILES,
    WIDTH,
};
pub use script::{run_cart, DEMO_SOURCE};

#[cfg(test)]
mod tests;
