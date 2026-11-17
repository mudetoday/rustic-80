use crate::ram::{
    self, poke4, Fill, BORDER, GAMEPADS, HEIGHT, KEYBOARD, MAP, MAP_HEIGHT, MAP_WIDTH, MOUSE,
    PALETTE, PALETTE_MAP, RAM_NIBBLES, RAM_SIZE, SPRITES, SWEETIE16, TILES, TILE_BYTES, WIDTH,
};

/// Host input sampled once per frame, before `TIC()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// 4 players × 8 buttons. Bit 0 is player 1 Up, matching `tic80_gamepad`.
    pub buttons: u32,
    /// Up to four simultaneous key codes (`tic_keycode`).
    pub keys: [u8; 4],
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_left: bool,
    pub mouse_middle: bool,
    pub mouse_right: bool,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

pub struct Machine {
    pub ram: Box<[u8; RAM_SIZE]>,
    flags: [u8; 512],
    pub(crate) clip_l: i32,
    pub(crate) clip_t: i32,
    pub(crate) clip_r: i32,
    pub(crate) clip_b: i32,
    frame: u64,
    buttons: u32,
    prev_buttons: u32,
    holds: [u32; 32],
    keys: [u8; 4],
    prev_keys: [u8; 4],
    pub traces: Vec<(String, u8)>,
    pub exited: bool,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_left: bool,
    pub mouse_middle: bool,
    pub mouse_right: bool,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

impl Machine {
    pub fn new() -> Self {
        let mut m = Self {
            ram: Box::new([0; RAM_SIZE]),
            flags: [0; 512],
            clip_l: 0,
            clip_t: 0,
            clip_r: WIDTH,
            clip_b: HEIGHT,
            frame: 0,
            buttons: 0,
            prev_buttons: 0,
            holds: [0; 32],
            keys: [0; 4],
            prev_keys: [0; 4],
            traces: Vec::new(),
            exited: false,
            mouse_x: 0,
            mouse_y: 0,
            mouse_left: false,
            mouse_middle: false,
            mouse_right: false,
            scroll_x: 0,
            scroll_y: 0,
        };
        m.reset();
        m
    }

    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.flags.fill(0);
        for (i, rgb) in SWEETIE16.iter().enumerate() {
            let o = PALETTE + i * 3;
            self.ram[o] = rgb[0];
            self.ram[o + 1] = rgb[1];
            self.ram[o + 2] = rgb[2];
        }
        for i in 0..16 {
            poke4(&mut self.ram[..], (PALETTE_MAP * 2) + i, i as u8);
        }
        self.clip_l = 0;
        self.clip_t = 0;
        self.clip_r = WIDTH;
        self.clip_b = HEIGHT;
        self.frame = 0;
        self.buttons = 0;
        self.prev_buttons = 0;
        self.holds = [0; 32];
        self.keys = [0; 4];
        self.prev_keys = [0; 4];
        self.traces.clear();
        self.exited = false;
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Milliseconds since the first `TIC`, stepping at 60 Hz.
    pub fn time_ms(&self) -> u32 {
        self.frame
            .saturating_sub(1)
            .saturating_mul(1000)
            .saturating_div(60) as u32
    }

    pub fn begin_frame(&mut self, input: &Input) {
        self.prev_buttons = self.buttons;
        self.buttons = input.buttons;
        self.ram[GAMEPADS..GAMEPADS + 4].copy_from_slice(&self.buttons.to_le_bytes());
        for i in 0..32 {
            if self.buttons & (1 << i) != 0 {
                self.holds[i] = self.holds[i].saturating_add(1);
            } else {
                self.holds[i] = 0;
            }
        }
        self.prev_keys = self.keys;
        self.keys = input.keys;
        self.ram[KEYBOARD..KEYBOARD + 4].copy_from_slice(&self.keys);
        self.mouse_x = input.mouse_x.clamp(0, 255);
        self.mouse_y = input.mouse_y.clamp(0, 255);
        self.mouse_left = input.mouse_left;
        self.mouse_middle = input.mouse_middle;
        self.mouse_right = input.mouse_right;
        self.scroll_x = input.scroll_x;
        self.scroll_y = input.scroll_y;
        self.ram[MOUSE] = self.mouse_x as u8;
        self.ram[MOUSE + 1] = self.mouse_y as u8;
        let mut btns = 0u16;
        if self.mouse_left {
            btns |= 1;
        }
        if self.mouse_middle {
            btns |= 2;
        }
        if self.mouse_right {
            btns |= 4;
        }
        self.ram[MOUSE + 2..MOUSE + 4].copy_from_slice(&btns.to_le_bytes());
        self.frame += 1;
    }

    pub fn btn(&self, index: i32) -> u32 {
        if index < 0 {
            self.buttons
        } else if (0..32).contains(&index) {
            self.buttons & (1 << index)
        } else {
            0
        }
    }

    pub fn btnp(&self, index: i32, hold: i32, period: i32) -> u32 {
        let now = self.buttons;
        let prev = self.prev_buttons;
        if index < 0 {
            return (!prev) & now;
        }
        if !(0..32).contains(&index) {
            return 0;
        }
        let mask = 1u32 << index;
        if hold < 0 || period < 0 {
            return ((!prev) & now) & mask;
        }
        let held = self.holds[index as usize];
        let previous = if held >= hold as u32 {
            if period != 0 && held % (period as u32) != 0 {
                prev
            } else {
                0
            }
        } else {
            prev
        };
        ((!previous) & now) & mask
    }

    pub fn key(&self, code: i32) -> bool {
        if code <= 0 {
            return self.keys.iter().any(|k| *k != 0);
        }
        let code = code as u8;
        self.keys.iter().any(|k| *k == code)
    }

    pub fn peek(&self, addr: i64) -> u8 {
        if addr < 0 || addr as usize >= RAM_SIZE {
            return 0;
        }
        self.ram[addr as usize]
    }

    pub fn poke(&mut self, addr: i64, value: u8) {
        if addr < 0 || addr as usize >= RAM_SIZE {
            return;
        }
        self.ram[addr as usize] = value;
    }

    pub fn peek4(&self, index: i64) -> u8 {
        if index < 0 || index as usize >= RAM_NIBBLES {
            return 0;
        }
        ram::peek4(&self.ram[..], index as usize)
    }

    pub fn poke4(&mut self, index: i64, value: u8) {
        if index < 0 || index as usize >= RAM_NIBBLES {
            return;
        }
        poke4(&mut self.ram[..], index as usize, value);
    }

    pub fn memcpy(&mut self, dest: i64, src: i64, len: i64) {
        if len <= 0 {
            return;
        }
        let len = len as usize;
        let dest = dest.max(0) as usize;
        let src = src.max(0) as usize;
        if dest >= RAM_SIZE || src >= RAM_SIZE {
            return;
        }
        let len = len.min(RAM_SIZE - dest).min(RAM_SIZE - src);
        self.ram.copy_within(src..src + len, dest);
    }

    pub fn memset(&mut self, addr: i64, value: u8, len: i64) {
        if len <= 0 || addr < 0 {
            return;
        }
        let addr = addr as usize;
        if addr >= RAM_SIZE {
            return;
        }
        let len = (len as usize).min(RAM_SIZE - addr);
        self.ram[addr..addr + len].fill(value);
    }

    pub fn mget(&self, x: i32, y: i32) -> u8 {
        if !(0..MAP_WIDTH).contains(&x) || !(0..MAP_HEIGHT).contains(&y) {
            return 0;
        }
        self.ram[MAP + (y as usize) * MAP_WIDTH as usize + x as usize]
    }

    pub fn mset(&mut self, x: i32, y: i32, value: u8) {
        if !(0..MAP_WIDTH).contains(&x) || !(0..MAP_HEIGHT).contains(&y) {
            return;
        }
        self.ram[MAP + (y as usize) * MAP_WIDTH as usize + x as usize] = value;
    }

    pub fn fget(&self, index: i32, flag: u8) -> bool {
        if !(0..512).contains(&index) || flag >= 8 {
            return false;
        }
        self.flags[index as usize] & (1 << flag) != 0
    }

    pub fn fset(&mut self, index: i32, flag: u8, value: bool) {
        if !(0..512).contains(&index) || flag >= 8 {
            return;
        }
        if value {
            self.flags[index as usize] |= 1 << flag;
        } else {
            self.flags[index as usize] &= !(1 << flag);
        }
    }

    pub fn palette_rgb(&self, index: u8) -> [u8; 3] {
        let i = (index & 15) as usize;
        let o = PALETTE + i * 3;
        [self.ram[o], self.ram[o + 1], self.ram[o + 2]]
    }

    pub fn border(&self) -> u8 {
        self.ram[BORDER] & 0x0f
    }

    pub fn map_color(&self, color: u8) -> u8 {
        ram::peek4(&self.ram[..], PALETTE_MAP * 2 + (color as usize & 15))
    }

    pub fn fill_from(&self, color: u32) -> Fill {
        let c1 = self.map_color(color as u8);
        let raw2 = (color >> 8) as u8;
        let c2 = if raw2 == ram::TRANSPARENT {
            ram::TRANSPARENT
        } else {
            self.map_color(raw2)
        };
        Fill {
            c1,
            c2,
            pattern: (color >> 16) as u16,
        }
    }

    pub fn pix(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT {
            return 0;
        }
        ram::peek4(&self.ram[..], (y as usize) * ram::WIDTH_USIZE + x as usize)
    }

    pub(crate) fn in_clip(&self, x: i32, y: i32) -> bool {
        x >= self.clip_l && y >= self.clip_t && x < self.clip_r && y < self.clip_b
    }

    pub(crate) fn put(&mut self, x: i32, y: i32, color: u8) {
        if !self.in_clip(x, y) {
            return;
        }
        poke4(
            &mut self.ram[..],
            (y as usize) * ram::WIDTH_USIZE + x as usize,
            color,
        );
    }

    pub(crate) fn put_fill(&mut self, x: i32, y: i32, fill: Fill) {
        if let Some(color) = fill.at(x, y) {
            self.put(x, y, color);
        }
    }

    pub fn clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let mut l = x;
        let mut t = y;
        let mut r = x.saturating_add(w);
        let mut b = y.saturating_add(h);
        l = l.clamp(0, WIDTH);
        t = t.clamp(0, HEIGHT);
        r = r.clamp(0, WIDTH);
        b = b.clamp(0, HEIGHT);
        if r < l {
            r = l;
        }
        if b < t {
            b = t;
        }
        self.clip_l = l;
        self.clip_t = t;
        self.clip_r = r;
        self.clip_b = b;
    }

    pub fn reset_clip(&mut self) {
        self.clip(0, 0, WIDTH, HEIGHT);
    }

    pub fn trace(&mut self, text: impl Into<String>, color: u8) {
        if self.traces.len() < 256 {
            self.traces.push((text.into(), color & 15));
        }
    }

    pub fn exit(&mut self) {
        self.exited = true;
    }

    /// RGBA8 of the 240×136 screen, using the live palette.
    pub fn framebuffer_rgba(&self) -> Vec<u8> {
        let mut out = vec![0u8; ram::WIDTH_USIZE * ram::HEIGHT_USIZE * 4];
        for y in 0..ram::HEIGHT_USIZE {
            for x in 0..ram::WIDTH_USIZE {
                let c = ram::peek4(&self.ram[..], y * ram::WIDTH_USIZE + x);
                let rgb = self.palette_rgb(c);
                let o = (y * ram::WIDTH_USIZE + x) * 4;
                out[o] = rgb[0];
                out[o + 1] = rgb[1];
                out[o + 2] = rgb[2];
                out[o + 3] = 255;
            }
        }
        out
    }

    pub(crate) fn tile_pix(&self, index: i32, x: i32, y: i32) -> u8 {
        if !(0..8).contains(&x) || !(0..8).contains(&y) {
            return 0;
        }
        let index = index.rem_euclid(512) as usize;
        let bank = index / 256;
        let id = index % 256;
        let base = if bank == 0 { TILES } else { SPRITES };
        let nibble = (base + id * TILE_BYTES) * 2 + (y as usize) * 8 + x as usize;
        ram::peek4(&self.ram[..], nibble)
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}
