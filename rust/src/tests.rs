use crate::ram::{peek4, poke4, GAMEPADS, MAP, PALETTE, RAM_SIZE, SPRITES, SWEETIE16, TILES};
use crate::{run_cart, Input, Machine, DEMO_SOURCE, HEIGHT, WIDTH};

#[test]
fn ram_matches_tic80_map() {
    assert_eq!(RAM_SIZE, 96 * 1024);
    assert_eq!(TILES, 0x4000);
    assert_eq!(SPRITES, 0x6000);
    assert_eq!(MAP, 0x8000);
    assert_eq!(GAMEPADS, 0xFF80);
    assert_eq!(0x8000 + (240 * 136), GAMEPADS);
    assert_eq!(WIDTH, 240);
    assert_eq!(HEIGHT, 136);
}

#[test]
fn sweetie_palette_is_loaded() {
    let m = Machine::new();
    assert_eq!(m.palette_rgb(0), SWEETIE16[0]);
    assert_eq!(m.palette_rgb(12), [0xf4, 0xf4, 0xf4]);
    assert_eq!(m.peek(PALETTE as i64), 0x1a);
    assert_eq!(m.map_color(7), 7);
}

#[test]
fn nibble_poke_roundtrip() {
    let mut ram = [0u8; 4];
    poke4(&mut ram, 0, 0x0a);
    poke4(&mut ram, 1, 0x0b);
    poke4(&mut ram, 3, 0x0c);
    assert_eq!(peek4(&ram, 0), 0x0a);
    assert_eq!(peek4(&ram, 1), 0x0b);
    assert_eq!(peek4(&ram, 2), 0);
    assert_eq!(peek4(&ram, 3), 0x0c);
    assert_eq!(ram[0], 0xba);
}

#[test]
fn cls_fills_screen_and_keeps_palette() {
    let mut m = Machine::new();
    m.cls(12);
    assert_eq!(m.pix(0, 0), 12);
    assert_eq!(m.pix(239, 135), 12);
    assert_eq!(m.pix(-1, 0), 0);
    assert_eq!(m.palette_rgb(0), SWEETIE16[0]);
}

#[test]
fn clip_limits_rect() {
    let mut m = Machine::new();
    m.clip(10, 10, 5, 4);
    m.cls(3);
    assert_eq!(m.pix(10, 10), 3);
    assert_eq!(m.pix(14, 13), 3);
    assert_eq!(m.pix(9, 10), 0);
    assert_eq!(m.pix(15, 10), 0);
    m.reset_clip();
    m.pix_set(0, 0, 1);
    assert_eq!(m.pix(0, 0), 1);
}

#[test]
fn hline_is_inclusive() {
    let mut m = Machine::new();
    m.line(2, 5, 6, 5, 9);
    assert_eq!(m.pix(1, 5), 0);
    for x in 2..=6 {
        assert_eq!(m.pix(x, 5), 9, "x={x}");
    }
    assert_eq!(m.pix(7, 5), 0);
}

#[test]
fn rect_and_border() {
    let mut m = Machine::new();
    m.rect(4, 4, 6, 5, 2);
    assert_eq!(m.pix(4, 4), 2);
    assert_eq!(m.pix(9, 8), 2);
    assert_eq!(m.pix(10, 4), 0);
    m.cls(0);
    m.rectb(4, 4, 6, 5, 7);
    assert_eq!(m.pix(4, 4), 7);
    assert_eq!(m.pix(9, 4), 7);
    assert_eq!(m.pix(5, 5), 0);
    assert_eq!(m.pix(4, 8), 7);
}

#[test]
fn pattern_checker() {
    let mut m = Machine::new();
    // bit n = (y & 3) * 4 + (x & 3). 0xA = bits 1 and 3.
    let color = 1u32 | (2 << 8) | (0xA << 16);
    m.rect(0, 0, 4, 1, color);
    assert_eq!(m.pix(0, 0), 2);
    assert_eq!(m.pix(1, 0), 1);
    assert_eq!(m.pix(2, 0), 2);
    assert_eq!(m.pix(3, 0), 1);
}

#[test]
fn sprite_respects_colorkey_and_bank() {
    let mut m = Machine::new();
    // Tile 2, pixel (1, 0) = 5, pixel (0, 0) = 0.
    let base = (TILES + 2 * 32) * 2;
    m.poke4((base + 1) as i64, 5);
    m.spr(2, 10, 12, &[0], 1, 0, 0, 1, 1);
    assert_eq!(m.pix(10, 12), 0);
    assert_eq!(m.pix(11, 12), 5);

    let fg = (SPRITES + 0 * 32) * 2;
    m.poke4((fg + 0) as i64, 8);
    m.spr(256, 20, 20, &[], 1, 0, 0, 1, 1);
    assert_eq!(m.pix(20, 20), 8);
}

#[test]
fn map_draws_tile() {
    let mut m = Machine::new();
    let base = (TILES + 3 * 32) * 2;
    m.poke4(base as i64, 11);
    m.mset(2, 4, 3);
    m.map(2, 4, 1, 1, 30, 40, &[0], 1);
    assert_eq!(m.pix(30, 40), 11);
    assert_eq!(m.mget(2, 4), 3);
    assert_eq!(m.mget(-1, 0), 0);
}

#[test]
fn print_draws_exclamation() {
    let mut m = Machine::new();
    let width = m.print("!", 0, 0, 12, false, 1);
    assert!(width > 0);
    assert!(m.pix(1, 0) == 12 || m.pix(2, 0) == 12);
}

#[test]
fn btn_edge_and_hold() {
    let mut m = Machine::new();
    let mut down = Input::default();
    down.buttons = 1;
    m.begin_frame(&down);
    assert_ne!(m.btn(0), 0);
    assert_ne!(m.btnp(0, -1, -1), 0);
    m.begin_frame(&down);
    assert_eq!(m.btnp(0, -1, -1), 0);
    assert_ne!(m.btn(0), 0);
    let up = Input::default();
    m.begin_frame(&up);
    assert_eq!(m.btn(0), 0);
    assert_eq!(m.btnp(0, -1, -1), 0);
}

#[test]
fn flags_and_memcpy() {
    let mut m = Machine::new();
    m.fset(4, 1, true);
    assert!(m.fget(4, 1));
    assert!(!m.fget(4, 0));
    m.poke(10, 9);
    m.poke(11, 8);
    m.memcpy(20, 10, 2);
    assert_eq!(m.peek(20), 9);
    assert_eq!(m.peek(21), 8);
    m.memset(20, 3, 2);
    assert_eq!(m.peek(20), 3);
}

#[test]
fn circ_covers_center() {
    let mut m = Machine::new();
    m.circ(20, 20, 4, 6);
    assert_eq!(m.pix(20, 20), 6);
    assert_eq!(m.pix(20, 16), 6);
    assert_eq!(m.pix(20, 15), 0);
}

#[test]
fn demo_cart_draws_a_frame() {
    let m = run_cart(DEMO_SOURCE, 1, |_| Input::default()).expect("demo");
    assert_eq!(m.pix(0, 2), 0);
    assert_eq!(m.pix(20, 2), 1);
    assert_eq!(m.pix(15 * 12, 2), 12);
    let mut white = 0;
    let mut ink = 0;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let c = m.pix(x, y);
            if c != 0 {
                ink += 1;
            }
            if c == 12 {
                white += 1;
            }
        }
    }
    assert!(ink > 100, "ink={ink}");
    assert!(white > 10, "white={white}");
    assert!(m.traces.iter().any(|(t, _)| t.contains("BOOT")));
}
