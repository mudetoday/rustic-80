use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::ExitCode;

use rustic80::{run_cart, Input, DEMO_SOURCE, HEIGHT, WIDTH};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("rustic80: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut source: Option<String> = None;
    let mut frames: u32 = 120;
    let mut png_path = PathBuf::from("frame.png");
    let mut use_demo = true;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!(
                    "\
rustic80 — TIC-80 core, Rust port

  rustic80                       run the built-in demo for 120 frames
  rustic80 run CART.lua          run a Lua cart (TIC / BOOT)
  rustic80 --frames N --png PATH

Buttons are not read from a keyboard yet; the demo animates on time()."
                );
                return Ok(());
            }
            "--frames" => {
                frames = args
                    .next()
                    .ok_or("--frames needs a number")?
                    .parse()
                    .map_err(|_| "--frames needs a number")?;
            }
            "--png" => {
                png_path = PathBuf::from(args.next().ok_or("--png needs a path")?);
            }
            "run" => {
                let path = args.next().ok_or("run needs a .lua cart")?;
                source = Some(std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?);
                use_demo = false;
            }
            "demo" => use_demo = true,
            other => return Err(format!("unknown argument {other} (try --help)")),
        }
    }

    let code = if use_demo {
        DEMO_SOURCE
    } else {
        source.as_deref().ok_or("no cart")?
    };
    let machine = run_cart(code, frames, |_frame| Input::default())?;
    write_png(&png_path, &machine.framebuffer_rgba())?;
    let ink = machine
        .framebuffer_rgba()
        .chunks(4)
        .filter(|p| p[0] != 0x1a || p[1] != 0x1c || p[2] != 0x2c)
        .count();
    println!(
        "ran {frames} frames, {WIDTH}x{HEIGHT}, {ink} pixels not background, wrote {}",
        png_path.display()
    );
    for (text, color) in &machine.traces {
        println!("trace({color}): {text}");
    }
    if ink == 0 {
        return Err("frame is empty".into());
    }
    Ok(())
}

fn write_png(path: &std::path::Path, rgba: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), WIDTH as u32, HEIGHT as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| format!("png: {e}"))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| format!("png: {e}"))?;
    Ok(())
}
