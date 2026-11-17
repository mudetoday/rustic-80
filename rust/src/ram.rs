//! TIC-80 address map. The live regions the core uses sit at the same
//! offsets as upstream `tic_ram` (see `src/tic.h`).

pub const WIDTH: i32 = 240;
pub const HEIGHT: i32 = 136;
pub const WIDTH_USIZE: usize = 240;
pub const HEIGHT_USIZE: usize = 136;

pub const VRAM_SIZE: usize = 16 * 1024;
pub const RAM_SIZE: usize = VRAM_SIZE + 80 * 1024;
pub const RAM_NIBBLES: usize = RAM_SIZE * 2;

// Addresses kept so the next modules (blit segment, cursor, font RAM) land on
// the same map even before anything reads them.
#[allow(dead_code)]
pub const SCREEN: usize = 0x0000;
#[allow(dead_code)]
pub const SCREEN_BYTES: usize = (WIDTH_USIZE * HEIGHT_USIZE) / 2;
pub const PALETTE: usize = 0x3FC0;
pub const PALETTE_MAP: usize = 0x3FF0;
pub const BORDER: usize = 0x3FF8;
#[allow(dead_code)]
pub const SCREEN_OFFSET: usize = 0x3FF9;
#[allow(dead_code)]
pub const CURSOR: usize = 0x3FFB;
#[allow(dead_code)]
pub const BLIT_SEGMENT: usize = 0x3FFC;

pub const TILES: usize = 0x4000;
pub const SPRITES: usize = 0x6000;
pub const TILE_BYTES: usize = 32;
#[allow(dead_code)]
pub const TILE_COUNT: usize = 256;

pub const MAP: usize = 0x8000;
pub const MAP_WIDTH: i32 = 240;
pub const MAP_HEIGHT: i32 = 136;
pub const MAP_SCREEN_W: i32 = 30;
pub const MAP_SCREEN_H: i32 = 17;

pub const GAMEPADS: usize = 0xFF80;
pub const MOUSE: usize = 0xFF84;
pub const KEYBOARD: usize = 0xFF88;

pub const TRANSPARENT: u8 = 255;

/// Sweetie-16, the palette `src/cart.c` copies into a fresh cart.
pub const SWEETIE16: [[u8; 3]; 16] = [
    [0x1a, 0x1c, 0x2c],
    [0x5d, 0x27, 0x5d],
    [0xb1, 0x3e, 0x53],
    [0xef, 0x7d, 0x57],
    [0xff, 0xcd, 0x75],
    [0xa7, 0xf0, 0x70],
    [0x38, 0xb7, 0x64],
    [0x25, 0x71, 0x79],
    [0x29, 0x36, 0x6f],
    [0x3b, 0x5d, 0xc9],
    [0x41, 0xa6, 0xf6],
    [0x73, 0xef, 0xf7],
    [0xf4, 0xf4, 0xf4],
    [0x94, 0xb0, 0xc2],
    [0x56, 0x6c, 0x86],
    [0x33, 0x3c, 0x57],
];

pub fn peek4(ram: &[u8], index: usize) -> u8 {
    if index >= RAM_NIBBLES {
        return 0;
    }
    (ram[index >> 1] >> ((index & 1) * 4)) & 0x0f
}

pub fn poke4(ram: &mut [u8], index: usize, value: u8) {
    if index >= RAM_NIBBLES {
        return;
    }
    let shift = (index & 1) * 4;
    let byte = &mut ram[index >> 1];
    *byte = (*byte & !(0x0f << shift)) | ((value & 0x0f) << shift);
}

/// Packed fill word: `c1 | c2 << 8 | pattern << 16`.
/// Pattern 0 is a solid `c1`. A set pattern bit draws `c1`, a clear bit
/// draws `c2`, and `c2 == 255` leaves the pixel alone.
#[derive(Clone, Copy)]
pub struct Fill {
    pub c1: u8,
    pub c2: u8,
    pub pattern: u16,
}

impl Fill {
    pub fn solid(c1: u8) -> Self {
        Self {
            c1,
            c2: c1,
            pattern: 0,
        }
    }

    pub fn at(self, x: i32, y: i32) -> Option<u8> {
        if self.pattern == 0 {
            return Some(self.c1);
        }
        let bit = ((y & 3) * 4 + (x & 3)) as u16;
        let color = if (self.pattern >> bit) & 1 == 1 {
            self.c1
        } else {
            self.c2
        };
        if color == TRANSPARENT {
            None
        } else {
            Some(color)
        }
    }
}
