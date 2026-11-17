use crate::machine::Machine;
use crate::ram::{Fill, HEIGHT, MAP_HEIGHT, MAP_SCREEN_H, MAP_SCREEN_W, MAP_WIDTH, WIDTH};

impl Machine {
    pub fn cls(&mut self, color: u8) {
        let color = self.map_color(color);
        let full =
            self.clip_l == 0 && self.clip_t == 0 && self.clip_r == WIDTH && self.clip_b == HEIGHT;
        if full {
            let byte = (color & 0x0f) | (color << 4);
            let n = (WIDTH as usize) * (HEIGHT as usize) / 2;
            self.ram[..n].fill(byte);
            return;
        }
        for y in self.clip_t..self.clip_b {
            self.hline_solid(self.clip_l, y, self.clip_r - self.clip_l, color);
        }
    }

    pub fn pix_set(&mut self, x: i32, y: i32, color: u8) {
        self.put(x, y, self.map_color(color));
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let fill = self.fill_from(color);
        if y0 == y1 {
            let (mut a, mut b) = (x0, x1);
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            self.hline(a, y0, b - a + 1, fill);
            return;
        }
        if x0 == x1 {
            let (mut a, mut b) = (y0, y1);
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            for y in a..=b {
                self.put_fill(x0, y, fill);
            }
            return;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (x0, y0, x1, y1);
        let steep = (y1 - y0).abs() > (x1 - x0).abs();
        if steep {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
        }
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }
        let dx = x1 - x0;
        let dy = (y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx / 2;
        let mut y = y0;
        for x in x0..=x1 {
            if steep {
                self.put_fill(y, x, fill);
            } else {
                self.put_fill(x, y, fill);
            }
            err -= dy;
            if err < 0 {
                y += sy;
                err += dx;
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        if w <= 0 || h <= 0 {
            return;
        }
        let fill = self.fill_from(color);
        for row in 0..h {
            self.hline(x, y + row, w, fill);
        }
    }

    pub fn rectb(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        if w <= 0 || h <= 0 {
            return;
        }
        let fill = self.fill_from(color);
        self.hline(x, y, w, fill);
        if h > 1 {
            self.hline(x, y + h - 1, w, fill);
        }
        for row in 0..h {
            self.put_fill(x, y + row, fill);
            if w > 1 {
                self.put_fill(x + w - 1, y + row, fill);
            }
        }
    }

    pub fn circ(&mut self, cx: i32, cy: i32, r: i32, color: u32) {
        if r < 0 {
            return;
        }
        let fill = self.fill_from(color);
        let mut x = r;
        let mut y = 0;
        let mut err = 1 - r;
        while x >= y {
            self.hline(cx - x, cy + y, x * 2 + 1, fill);
            if y != 0 {
                self.hline(cx - x, cy - y, x * 2 + 1, fill);
            }
            if y != x {
                self.hline(cx - y, cy + x, y * 2 + 1, fill);
                self.hline(cx - y, cy - x, y * 2 + 1, fill);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    pub fn circb(&mut self, cx: i32, cy: i32, r: i32, color: u32) {
        if r < 0 {
            return;
        }
        if r == 0 {
            self.pix_set(cx, cy, color as u8);
            return;
        }
        let fill = self.fill_from(color);
        let mut x = r;
        let mut y = 0;
        let mut err = 1 - r;
        while x >= y {
            self.put_fill(cx + x, cy + y, fill);
            self.put_fill(cx + y, cy + x, fill);
            self.put_fill(cx - y, cy + x, fill);
            self.put_fill(cx - x, cy + y, fill);
            self.put_fill(cx - x, cy - y, fill);
            self.put_fill(cx - y, cy - x, fill);
            self.put_fill(cx + y, cy - x, fill);
            self.put_fill(cx + x, cy - y, fill);
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    pub fn tri(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: u32) {
        let fill = self.fill_from(color);
        let min_x = x1.min(x2).min(x3).max(self.clip_l);
        let max_x = x1.max(x2).max(x3).min(self.clip_r - 1);
        let min_y = y1.min(y2).min(y3).max(self.clip_t);
        let max_y = y1.max(y2).max(y3).min(self.clip_b - 1);
        if min_x > max_x || min_y > max_y {
            return;
        }
        let area = edge(x1, y1, x2, y2, x3, y3);
        if area == 0 {
            return;
        }
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let w0 = edge(x2, y2, x3, y3, x, y);
                let w1 = edge(x3, y3, x1, y1, x, y);
                let w2 = edge(x1, y1, x2, y2, x, y);
                let inside = if area > 0 {
                    w0 >= 0 && w1 >= 0 && w2 >= 0
                } else {
                    w0 <= 0 && w1 <= 0 && w2 <= 0
                };
                if inside {
                    self.put_fill(x, y, fill);
                }
            }
        }
    }

    pub fn spr(
        &mut self,
        index: i32,
        x: i32,
        y: i32,
        trans: &[u8],
        scale: i32,
        flip: u8,
        rotate: u8,
        w: i32,
        h: i32,
    ) {
        if index < 0 || w <= 0 || h <= 0 {
            return;
        }
        let scale = scale.max(1);
        let flip = flip & 3;
        let rotate = rotate & 3;
        if w == 1 && h == 1 {
            self.draw_tile(index, x, y, trans, scale, flip, rotate);
            return;
        }
        let step = 8 * scale;
        const COLS: i32 = 16;
        for i in 0..w {
            for j in 0..h {
                let mut mx = i;
                let mut my = j;
                if flip == 1 || flip == 3 {
                    mx = w - 1 - i;
                }
                if flip == 2 || flip == 3 {
                    my = h - 1 - j;
                }
                if rotate == 2 {
                    mx = w - 1 - mx;
                    my = h - 1 - my;
                } else if rotate == 1 {
                    if flip == 0 || flip == 3 {
                        my = h - 1 - my;
                    } else {
                        mx = w - 1 - mx;
                    }
                } else if rotate == 3 {
                    if flip == 0 || flip == 3 {
                        mx = w - 1 - mx;
                    } else {
                        my = h - 1 - my;
                    }
                }
                let id = index + mx + my * COLS;
                if rotate == 0 || rotate == 2 {
                    self.draw_tile(id, x + i * step, y + j * step, trans, scale, flip, rotate);
                } else {
                    self.draw_tile(id, x + j * step, y + i * step, trans, scale, flip, rotate);
                }
            }
        }
    }

    pub fn map(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sx: i32,
        sy: i32,
        trans: &[u8],
        scale: i32,
    ) {
        let w = if w == 0 { MAP_SCREEN_W } else { w };
        let h = if h == 0 { MAP_SCREEN_H } else { h };
        if w < 0 || h < 0 {
            return;
        }
        let scale = scale.max(1);
        let size = 8 * scale;
        for j in 0..h {
            for i in 0..w {
                let tx = (x + i).rem_euclid(MAP_WIDTH);
                let ty = (y + j).rem_euclid(MAP_HEIGHT);
                let tile = self.mget(tx, ty) as i32;
                self.spr(tile, sx + i * size, sy + j * size, trans, scale, 0, 0, 1, 1);
            }
        }
    }

    fn hline(&mut self, x: i32, y: i32, width: i32, fill: Fill) {
        if width <= 0 || y < self.clip_t || y >= self.clip_b {
            return;
        }
        let left = x.max(self.clip_l);
        let right = (x + width).min(self.clip_r);
        if left >= right {
            return;
        }
        if fill.pattern == 0 {
            self.hline_solid(left, y, right - left, fill.c1);
            return;
        }
        for px in left..right {
            self.put_fill(px, y, fill);
        }
    }

    fn hline_solid(&mut self, x: i32, y: i32, width: i32, color: u8) {
        if width <= 0 || y < 0 || y >= HEIGHT {
            return;
        }
        let left = x.max(0);
        let right = (x + width).min(WIDTH);
        if left >= right {
            return;
        }
        // Caller already clipped against the clip rect. This path only
        // clamps to the screen so a full `cls` can memset.
        for px in left..right {
            self.put(px, y, color);
        }
    }

    fn draw_tile(
        &mut self,
        index: i32,
        x: i32,
        y: i32,
        trans: &[u8],
        scale: i32,
        flip: u8,
        rotate: u8,
    ) {
        let mut mapping = [0u8; 16];
        for i in 0..16 {
            mapping[i] = self.map_color(i as u8);
        }
        for c in trans {
            if *c < 16 {
                mapping[*c as usize] = crate::ram::TRANSPARENT;
            }
        }
        let mut orientation = (flip & 3) as u32;
        if rotate == 1 {
            orientation ^= 1;
        } else if rotate == 2 {
            orientation ^= 3;
        } else if rotate == 3 {
            orientation ^= 2;
        }
        if rotate == 1 || rotate == 3 {
            orientation |= 4;
        }
        if x + 8 * scale <= self.clip_l
            || y + 8 * scale <= self.clip_t
            || x >= self.clip_r
            || y >= self.clip_b
        {
            return;
        }
        for py in 0..8 {
            for px in 0..8 {
                let mut ix = if orientation & 1 != 0 { 7 - px } else { px };
                let mut iy = if orientation & 2 != 0 { 7 - py } else { py };
                if orientation & 4 != 0 {
                    std::mem::swap(&mut ix, &mut iy);
                }
                let raw = self.tile_pix(index, ix, iy);
                let color = mapping[raw as usize];
                if color == crate::ram::TRANSPARENT {
                    continue;
                }
                let dx = x + px * scale;
                let dy = y + py * scale;
                if scale == 1 {
                    self.put(dx, dy, color);
                } else {
                    let solid = Fill::solid(color);
                    for row in 0..scale {
                        self.hline(dx, dy + row, scale, solid);
                    }
                }
            }
        }
    }
}

fn edge(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i64 {
    (px as i64 - ax as i64) * (by as i64 - ay as i64)
        - (py as i64 - ay as i64) * (bx as i64 - ax as i64)
}
