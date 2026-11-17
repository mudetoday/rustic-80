//! Rustic-80 core: a Rust port of the TIC-80 fantasy computer.
//!
//! This crate is the first slice of the port — the 96K RAM map, the draw
//! API, input, the system font, and a sandboxed Lua cart runner. The C
//! studio in the repository root is still the full TIC-80 until each
//! module moves over.
//!
//! The `lua` feature (on by default) embeds Lua for the native player.
//! `wasm32-unknown-unknown` builds without it and exports the same machine
//! as a C ABI the browser drives.

mod draw;
mod font;
mod machine;
mod ram;
mod text;

#[cfg(feature = "lua")]
mod script;

#[cfg(target_arch = "wasm32")]
mod wasm_api;

pub use machine::{Input, Machine};
pub use ram::{
    GAMEPADS, HEIGHT, MAP, MAP_HEIGHT, MAP_WIDTH, PALETTE, RAM_SIZE, SPRITES, SWEETIE16, TILES,
    WIDTH,
};

#[cfg(feature = "lua")]
pub use script::{run_cart, DEMO_SOURCE};

#[cfg(test)]
mod tests;
