use crate::font::FONT;
use crate::machine::Machine;

impl Machine {
    /// `print`. `fixed` advances a constant 6px; otherwise the glyph is trimmed.
    /// Returns the width of the longest line, in pixels.
    pub fn print(&mut self, text: &str, x: i32, y: i32, color: u8, fixed: bool, scale: i32) -> i32 {
        let scale = scale.max(1);
        let color = self.map_color(color);
        let advance_empty = if fixed { 6 } else { 4 };
        let line_h = 6 * scale;
        let mut pos = x;
        let mut max = x;
        let mut cursor_y = y;
        for ch in text.chars() {
            if ch == '\n' {
                if pos > max {
                    max = pos;
                }
                pos = x;
                cursor_y += line_h;
                continue;
            }
            let code = if (ch as u32) < 128 { ch as u8 } else { 0 };
            let (start, end) = if fixed { (0, 8) } else { glyph_span(code) };
            let size = end - start;
            for col in start..end {
                for row in 0..8 {
                    if glyph_bit(code, col, row) {
                        let px = pos + (col - start) * scale;
                        let py = cursor_y + row * scale;
                        if scale == 1 {
                            self.put(px, py, color);
                        } else {
                            for sy in 0..scale {
                                for sx in 0..scale {
                                    self.put(px + sx, py + sy, color);
                                }
                            }
                        }
                    }
                }
            }
            let step = if !fixed && size > 0 {
                size + 1
            } else {
                advance_empty
            };
            pos += step * scale;
        }
        if pos > max {
            pos - x
        } else {
            max - x
        }
    }
}

fn glyph_bit(ch: u8, x: i32, y: i32) -> bool {
    if ch >= 127 || !(0..8).contains(&x) || !(0..8).contains(&y) {
        return false;
    }
    (FONT[ch as usize][y as usize] >> x) & 1 == 1
}

/// Inclusive-exclusive column span of ink, matching `drawChar` when `fixed` is false.
fn glyph_span(ch: u8) -> (i32, i32) {
    let mut start = 0;
    let mut end = 8;
    while start < 8 {
        let mut ink = false;
        for y in 0..8 {
            if glyph_bit(ch, start, y) {
                ink = true;
                break;
            }
        }
        if ink {
            break;
        }
        start += 1;
    }
    while end > start {
        let mut ink = false;
        for y in 0..8 {
            if glyph_bit(ch, end - 1, y) {
                ink = true;
                break;
            }
        }
        if ink {
            break;
        }
        end -= 1;
    }
    (start, end)
}
