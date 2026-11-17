//! C ABI for the browser build (`wasm32-unknown-unknown`, no Lua).
//! The page owns the cart; these calls are the TIC-80 machine.

use std::cell::UnsafeCell;

use crate::machine::{Input, Machine};
use crate::ram::{HEIGHT, WIDTH};

struct State {
    machine: Machine,
    rgba: Box<[u8]>,
    scratch: [u8; 16384],
}

struct Slot(UnsafeCell<Option<State>>);
unsafe impl Sync for Slot {}

static STATE: Slot = Slot(UnsafeCell::new(None));

fn state() -> &'static mut State {
    let slot = unsafe { &mut *STATE.0.get() };
    if slot.is_none() {
        *slot = Some(State {
            machine: Machine::new(),
            rgba: vec![0; (WIDTH as usize) * (HEIGHT as usize) * 4].into_boxed_slice(),
            scratch: [0; 16384],
        });
    }
    slot.as_mut().unwrap()
}

fn trans_from_mask(mask: u32) -> [u8; 16] {
    let mut buf = [0u8; 16];
    let mut n = 0;
    for i in 0..16u32 {
        if mask & (1 << i) != 0 {
            buf[n] = i as u8;
            n += 1;
        }
    }
    // Length is encoded in unused high sentinel: callers pass `n` separately.
    let _ = n;
    buf
}

fn trans_len(mask: u32) -> usize {
    mask.count_ones() as usize
}

#[no_mangle]
pub extern "C" fn r80_reset() {
    state().machine.reset();
}

#[no_mangle]
pub extern "C" fn r80_begin_frame(
    buttons: u32,
    k0: u32,
    k1: u32,
    k2: u32,
    k3: u32,
    mx: i32,
    my: i32,
    ml: u32,
    mm: u32,
    mr: u32,
    sx: i32,
    sy: i32,
) {
    let input = Input {
        buttons,
        keys: [k0 as u8, k1 as u8, k2 as u8, k3 as u8],
        mouse_x: mx,
        mouse_y: my,
        mouse_left: ml != 0,
        mouse_middle: mm != 0,
        mouse_right: mr != 0,
        scroll_x: sx,
        scroll_y: sy,
    };
    state().machine.begin_frame(&input);
}

#[no_mangle]
pub extern "C" fn r80_cls(color: u32) {
    state().machine.cls(color as u8);
}

#[no_mangle]
pub extern "C" fn r80_pix(x: i32, y: i32) -> u32 {
    state().machine.pix(x, y) as u32
}

#[no_mangle]
pub extern "C" fn r80_pix_set(x: i32, y: i32, color: u32) {
    state().machine.pix_set(x, y, color as u8);
}

#[no_mangle]
pub extern "C" fn r80_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    state().machine.line(x0, y0, x1, y1, color);
}

#[no_mangle]
pub extern "C" fn r80_rect(x: i32, y: i32, w: i32, h: i32, color: u32) {
    state().machine.rect(x, y, w, h, color);
}

#[no_mangle]
pub extern "C" fn r80_rectb(x: i32, y: i32, w: i32, h: i32, color: u32) {
    state().machine.rectb(x, y, w, h, color);
}

#[no_mangle]
pub extern "C" fn r80_circ(x: i32, y: i32, r: i32, color: u32) {
    state().machine.circ(x, y, r, color);
}

#[no_mangle]
pub extern "C" fn r80_circb(x: i32, y: i32, r: i32, color: u32) {
    state().machine.circb(x, y, r, color);
}

#[no_mangle]
pub extern "C" fn r80_tri(x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, color: u32) {
    state().machine.tri(x1, y1, x2, y2, x3, y3, color);
}

#[no_mangle]
pub extern "C" fn r80_spr(
    id: i32,
    x: i32,
    y: i32,
    mask: u32,
    scale: i32,
    flip: u32,
    rotate: u32,
    w: i32,
    h: i32,
) {
    let buf = trans_from_mask(mask);
    let n = trans_len(mask).min(16);
    state()
        .machine
        .spr(id, x, y, &buf[..n], scale, flip as u8, rotate as u8, w, h);
}

#[no_mangle]
pub extern "C" fn r80_map(x: i32, y: i32, w: i32, h: i32, sx: i32, sy: i32, mask: u32, scale: i32) {
    let buf = trans_from_mask(mask);
    let n = trans_len(mask).min(16);
    state().machine.map(x, y, w, h, sx, sy, &buf[..n], scale);
}

#[no_mangle]
pub extern "C" fn r80_print(len: u32, x: i32, y: i32, color: u32, fixed: u32, scale: i32) -> i32 {
    let s = state();
    let n = (len as usize).min(s.scratch.len());
    let text = String::from_utf8_lossy(&s.scratch[..n]).into_owned();
    s.machine.print(&text, x, y, color as u8, fixed != 0, scale)
}

#[no_mangle]
pub extern "C" fn r80_scratch() -> *mut u8 {
    state().scratch.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn r80_btn(id: i32) -> u32 {
    state().machine.btn(id)
}

#[no_mangle]
pub extern "C" fn r80_btnp(id: i32, hold: i32, period: i32) -> u32 {
    state().machine.btnp(id, hold, period)
}

#[no_mangle]
pub extern "C" fn r80_key(code: i32) -> u32 {
    u32::from(state().machine.key(code))
}

#[no_mangle]
pub extern "C" fn r80_mget(x: i32, y: i32) -> u32 {
    state().machine.mget(x, y) as u32
}

#[no_mangle]
pub extern "C" fn r80_mset(x: i32, y: i32, value: u32) {
    state().machine.mset(x, y, value as u8);
}

#[no_mangle]
pub extern "C" fn r80_peek(addr: i32) -> u32 {
    state().machine.peek(addr as i64) as u32
}

#[no_mangle]
pub extern "C" fn r80_poke(addr: i32, value: u32) {
    state().machine.poke(addr as i64, value as u8);
}

#[no_mangle]
pub extern "C" fn r80_peek4(addr: i32) -> u32 {
    state().machine.peek4(addr as i64) as u32
}

#[no_mangle]
pub extern "C" fn r80_poke4(addr: i32, value: u32) {
    state().machine.poke4(addr as i64, value as u8);
}

#[no_mangle]
pub extern "C" fn r80_fget(index: i32, flag: u32) -> u32 {
    u32::from(state().machine.fget(index, flag as u8))
}

#[no_mangle]
pub extern "C" fn r80_fset(index: i32, flag: u32, value: u32) {
    state().machine.fset(index, flag as u8, value != 0);
}

#[no_mangle]
pub extern "C" fn r80_clip(x: i32, y: i32, w: i32, h: i32) {
    state().machine.clip(x, y, w, h);
}

#[no_mangle]
pub extern "C" fn r80_time() -> u32 {
    state().machine.time_ms()
}

#[no_mangle]
pub extern "C" fn r80_exit() {
    state().machine.exit();
}

#[no_mangle]
pub extern "C" fn r80_exited() -> u32 {
    u32::from(state().machine.exited)
}

#[no_mangle]
pub extern "C" fn r80_mouse_x() -> i32 {
    state().machine.mouse_x
}

#[no_mangle]
pub extern "C" fn r80_mouse_y() -> i32 {
    state().machine.mouse_y
}

#[no_mangle]
pub extern "C" fn r80_mouse_btn() -> u32 {
    let m = &state().machine;
    u32::from(m.mouse_left) | (u32::from(m.mouse_middle) << 1) | (u32::from(m.mouse_right) << 2)
}

#[no_mangle]
pub extern "C" fn r80_scroll_x() -> i32 {
    state().machine.scroll_x
}

#[no_mangle]
pub extern "C" fn r80_scroll_y() -> i32 {
    state().machine.scroll_y
}

#[no_mangle]
pub extern "C" fn r80_memcpy(dest: i32, src: i32, len: i32) {
    state().machine.memcpy(dest as i64, src as i64, len as i64);
}

#[no_mangle]
pub extern "C" fn r80_memset(addr: i32, value: u32, len: i32) {
    state().machine.memset(addr as i64, value as u8, len as i64);
}

/// Packs the 240×136 screen into the RGBA buffer and returns its pointer.
#[no_mangle]
pub extern "C" fn r80_blit() -> *mut u8 {
    let s = state();
    let frame = s.machine.framebuffer_rgba();
    s.rgba.copy_from_slice(&frame);
    s.rgba.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn r80_width() -> u32 {
    WIDTH as u32
}

#[no_mangle]
pub extern "C" fn r80_height() -> u32 {
    HEIGHT as u32
}
