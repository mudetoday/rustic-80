# rustic80

First slice of the Rust port of [TIC-80](https://github.com/nesbox/TIC-80).

The C studio at the repository root is still upstream TIC-80. This crate is the machine it will be replaced by, starting at the core:

- 240×136, 16-color screen and the Sweetie-16 palette
- The same RAM map the wiki documents (`0x0000` screen, `0x4000` tiles, `0x6000` sprites, `0x8000` map, `0xFF80` gamepads)
- `cls` `pix` `line` `rect` `rectb` `circ` `circb` `tri` `spr` `map` `print` `clip`
- `peek`/`poke`/`peek4`/`poke4`, `mget`/`mset`, `fget`/`fset`, `memcpy`/`memset`
- `btn`/`btnp`/`key`/`mouse`, `trace`, `time`, `exit`
- Packed fill colors (`c1 | c2 << 8 | pattern << 16`)
- A Lua cart (`BOOT` / `TIC`) on a sandbox without `io` or `os`

Not here yet: the code/sprite/map/sfx editors, sound, `ttri`, every language binding, and `.tic` cartridges. Circles and triangles are the usual integer rasterizers, not a pixel-identical copy of `draw.c`.

```bash
cd rust
cargo test
cargo run -- --frames 60 --png frame.png
cargo run -- run carts/demo.lua --frames 60 --png frame.png

The same machine builds for the browser without embedded Lua (`wasm32-unknown-unknown`, `--no-default-features`). The page drives it.
```bash
cargo build --lib --release --target wasm32-unknown-unknown --no-default-features
```
```
