use std::cell::RefCell;
use std::rc::Rc;

use mlua::{Lua, LuaOptions, StdLib, Value};

use crate::machine::{Input, Machine};

const PRELUDE: &str = r#"
function cls(color) _cls(color or 0) end
function pix(x, y, color)
  if color == nil then return _pix(x, y) else _pix(x, y, color) end
end
function line(x0, y0, x1, y1, color) _line(x0, y0, x1, y1, color or 15) end
function rect(x, y, w, h, color) _rect(x, y, w, h, color or 15) end
function rectb(x, y, w, h, color) _rectb(x, y, w, h, color or 15) end
function circ(x, y, r, color) _circ(x, y, r, color or 15) end
function circb(x, y, r, color) _circb(x, y, r, color or 15) end
function tri(x1, y1, x2, y2, x3, y3, color)
  _tri(x1, y1, x2, y2, x3, y3, color or 15)
end
function spr(id, x, y, colorkey, scale, flip, rotate, w, h)
  _spr(id or 0, x or 0, y or 0, colorkey, scale or 1, flip or 0, rotate or 0, w or 1, h or 1)
end
function map(x, y, w, h, sx, sy, colorkey, scale)
  _map(x or 0, y or 0, w or 30, h or 17, sx or 0, sy or 0, colorkey, scale or 1)
end
function print(text, x, y, color, fixed, scale)
  return _print(tostring(text), x or 0, y or 0, color or 15, not not fixed, scale or 1)
end
function btn(id)
  if id == nil then return _btn(-1) else return _btn(id) ~= 0 end
end
function btnp(id, hold, period)
  if id == nil then return _btnp(-1, hold or -1, period or -1) end
  return _btnp(id, hold or -1, period or -1) ~= 0
end
function key(k)
  if k == nil then return _key(0) else return _key(k) end
end
function mget(x, y) return _mget(x, y) end
function mset(x, y, v) _mset(x, y, v) end
function peek(a) return _peek(a) end
function poke(a, v) _poke(a, v) end
function peek4(a) return _peek4(a) end
function poke4(a, v) _poke4(a, v) end
function fget(i, f) return _fget(i, f or 0) end
function fset(i, f, v) _fset(i, f, not not v) end
function clip(x, y, w, h)
  if x == nil then _clip(0, 0, 240, 136) else _clip(x, y, w, h) end
end
function time() return _time() end
function tstamp() return _tstamp() end
function trace(msg, color) _trace(tostring(msg), color or 15) end
function exit() _exit() end
function mouse() return _mouse() end
function memcpy(dest, src, len) _memcpy(dest, src, len) end
function memset(addr, val, len) _memset(addr, val, len) end
"#;

pub const DEMO_SOURCE: &str = include_str!("../carts/demo.lua");

/// Run a Lua cart for `frames` frames. `input` is sampled at the start of each frame.
pub fn run_cart(
    source: &str,
    frames: u32,
    mut input: impl FnMut(u32) -> Input,
) -> Result<Machine, String> {
    let lua = Lua::new_with(
        StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH,
        LuaOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    let machine = Rc::new(RefCell::new(Machine::new()));
    bind(&lua, &machine)?;
    lua.load(PRELUDE)
        .set_name("prelude")
        .exec()
        .map_err(|e| e.to_string())?;
    lua.load(source)
        .set_name("cart")
        .exec()
        .map_err(|e| e.to_string())?;

    let boot: Option<mlua::Function> = lua.globals().get("BOOT").ok();
    if let Some(boot) = boot {
        boot.call::<()>(()).map_err(|e| format!("BOOT: {e}"))?;
    }
    let tic: mlua::Function = lua
        .globals()
        .get("TIC")
        .map_err(|_| "cart has no TIC() function".to_string())?;

    for frame in 0..frames {
        {
            let mut m = machine.borrow_mut();
            if m.exited {
                break;
            }
            let sample = input(frame);
            m.begin_frame(&sample);
        }
        tic.call::<()>(()).map_err(|e| format!("TIC: {e}"))?;
    }

    drop(lua);
    let m = Rc::try_unwrap(machine)
        .map_err(|_| "cart still holds the machine".to_string())?
        .into_inner();
    Ok(m)
}

fn bind(lua: &Lua, machine: &Rc<RefCell<Machine>>) -> Result<(), String> {
    let g = lua.globals();
    let bind_fn = |name: &str, f: mlua::Function| -> Result<(), String> {
        g.set(name, f).map_err(|e| e.to_string())
    };

    let m = machine.clone();
    bind_fn(
        "_cls",
        lua.create_function(move |_, color: i64| {
            m.borrow_mut().cls(color as u8);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_pix",
        lua.create_function(move |_, (x, y, color): (i64, i64, Option<i64>)| {
            let mut mac = m.borrow_mut();
            if let Some(color) = color {
                mac.pix_set(x as i32, y as i32, color as u8);
                Ok(None)
            } else {
                Ok(Some(mac.pix(x as i32, y as i32) as i64))
            }
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_line",
        lua.create_function(
            move |_, (x0, y0, x1, y1, color): (f64, f64, f64, f64, i64)| {
                m.borrow_mut()
                    .line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, color as u32);
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_rect",
        lua.create_function(move |_, (x, y, w, h, color): (i64, i64, i64, i64, i64)| {
            m.borrow_mut()
                .rect(x as i32, y as i32, w as i32, h as i32, color as u32);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_rectb",
        lua.create_function(move |_, (x, y, w, h, color): (i64, i64, i64, i64, i64)| {
            m.borrow_mut()
                .rectb(x as i32, y as i32, w as i32, h as i32, color as u32);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_circ",
        lua.create_function(move |_, (x, y, r, color): (i64, i64, i64, i64)| {
            m.borrow_mut()
                .circ(x as i32, y as i32, r as i32, color as u32);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_circb",
        lua.create_function(move |_, (x, y, r, color): (i64, i64, i64, i64)| {
            m.borrow_mut()
                .circb(x as i32, y as i32, r as i32, color as u32);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_tri",
        lua.create_function(
            move |_, (x1, y1, x2, y2, x3, y3, color): (f64, f64, f64, f64, f64, f64, i64)| {
                m.borrow_mut().tri(
                    x1 as i32,
                    y1 as i32,
                    x2 as i32,
                    y2 as i32,
                    x3 as i32,
                    y3 as i32,
                    color as u32,
                );
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_spr",
        lua.create_function(
            move |lua,
                  (id, x, y, key, scale, flip, rotate, w, h): (
                i64,
                i64,
                i64,
                Value,
                i64,
                i64,
                i64,
                i64,
                i64,
            )| {
                let trans = transparent(lua, key)?;
                m.borrow_mut().spr(
                    id as i32,
                    x as i32,
                    y as i32,
                    &trans,
                    scale as i32,
                    flip as u8,
                    rotate as u8,
                    w as i32,
                    h as i32,
                );
                Ok(())
            },
        )
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_map",
        lua
            .create_function(
                move |lua,
                      (x, y, w, h, sx, sy, key, scale): (
                    i64,
                    i64,
                    i64,
                    i64,
                    i64,
                    i64,
                    Value,
                    i64,
                )| {
                    let trans = transparent(lua, key)?;
                    m.borrow_mut().map(
                        x as i32,
                        y as i32,
                        w as i32,
                        h as i32,
                        sx as i32,
                        sy as i32,
                        &trans,
                        scale as i32,
                    );
                    Ok(())
                },
            )
            .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_print",
        lua.create_function(
            move |_, (text, x, y, color, fixed, scale): (String, i64, i64, i64, bool, i64)| {
                let width = m.borrow_mut().print(
                    &text,
                    x as i32,
                    y as i32,
                    color as u8,
                    fixed,
                    scale as i32,
                );
                Ok(width as i64)
            },
        )
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_btn",
        lua.create_function(move |_, id: i64| Ok(m.borrow().btn(id as i32) as i64))
            .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_btnp",
        lua.create_function(move |_, (id, hold, period): (i64, i64, i64)| {
            Ok(m.borrow().btnp(id as i32, hold as i32, period as i32) as i64)
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_key",
        lua.create_function(move |_, code: i64| Ok(m.borrow().key(code as i32)))
            .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_mget",
        lua.create_function(move |_, (x, y): (i64, i64)| {
            Ok(m.borrow().mget(x as i32, y as i32) as i64)
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_mset",
        lua.create_function(move |_, (x, y, v): (i64, i64, i64)| {
            m.borrow_mut().mset(x as i32, y as i32, v as u8);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_peek",
        lua.create_function(move |_, addr: i64| Ok(m.borrow().peek(addr) as i64))
            .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_poke",
        lua.create_function(move |_, (addr, value): (i64, i64)| {
            m.borrow_mut().poke(addr, value as u8);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_peek4",
        lua.create_function(move |_, addr: i64| Ok(m.borrow().peek4(addr) as i64))
            .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_poke4",
        lua.create_function(move |_, (addr, value): (i64, i64)| {
            m.borrow_mut().poke4(addr, value as u8);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_fget",
        lua.create_function(move |_, (index, flag): (i64, i64)| {
            Ok(m.borrow().fget(index as i32, flag as u8))
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_fset",
        lua.create_function(move |_, (index, flag, value): (i64, i64, bool)| {
            m.borrow_mut().fset(index as i32, flag as u8, value);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_clip",
        lua.create_function(move |_, (x, y, w, h): (i64, i64, i64, i64)| {
            m.borrow_mut().clip(x as i32, y as i32, w as i32, h as i32);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_time",
        lua.create_function(move |_, (): ()| Ok(m.borrow().time_ms() as i64))
            .map_err(|e| e.to_string())?,
    )?;

    bind_fn(
        "_tstamp",
        lua.create_function(|_, (): ()| {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            Ok(secs)
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_trace",
        lua.create_function(move |_, (text, color): (String, i64)| {
            m.borrow_mut().trace(text, color as u8);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_exit",
        lua.create_function(move |_, (): ()| {
            m.borrow_mut().exit();
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_mouse",
        lua.create_function(move |_, (): ()| {
            let mac = m.borrow();
            Ok((
                mac.mouse_x as i64,
                mac.mouse_y as i64,
                mac.mouse_left,
                mac.mouse_middle,
                mac.mouse_right,
                mac.scroll_x as i64,
                mac.scroll_y as i64,
            ))
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_memcpy",
        lua.create_function(move |_, (dest, src, len): (i64, i64, i64)| {
            m.borrow_mut().memcpy(dest, src, len);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    let m = machine.clone();
    bind_fn(
        "_memset",
        lua.create_function(move |_, (addr, val, len): (i64, i64, i64)| {
            m.borrow_mut().memset(addr, val as u8, len);
            Ok(())
        })
        .map_err(|e| e.to_string())?,
    )?;

    Ok(())
}

fn transparent(_lua: &Lua, key: Value) -> mlua::Result<Vec<u8>> {
    match key {
        Value::Nil => Ok(Vec::new()),
        Value::Integer(i) => Ok(one_key(i)),
        Value::Number(n) => Ok(one_key(n as i64)),
        Value::Table(t) => {
            let mut out = Vec::new();
            for i in 1..=16 {
                match t.raw_get::<i64>(i)? {
                    n if (0..16).contains(&n) => out.push(n as u8),
                    _ => break,
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn one_key(i: i64) -> Vec<u8> {
    if (0..16).contains(&i) {
        vec![i as u8]
    } else {
        Vec::new()
    }
}
