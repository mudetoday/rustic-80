# Rust port

`rust/` is where TIC-80 is being rewritten in Rust. The C sources stay in place until a module has a Rust replacement that passes its tests.

What runs today is the machine core plus a Lua player:

```bash
cd rust
cargo test
cargo run --release -- --frames 120 --png frame.png
```

`cargo run` plays `rust/carts/demo.lua` headless and writes a 240×136 PNG. There is no window yet.

The public draw and memory API follows TIC-80 (`src/tic.h`, `src/core/draw.c`). Later steps are the `.tic` loader, the sound chip, `ttri`, and the studio screens.
