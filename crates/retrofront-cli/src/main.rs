use retrofront_core::{overlay::parse_retro_id, FrontendConfig, RetroHost};
use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("retrofront-linux: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let core = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: retrofront-linux <core_libretro.so> [content] [frames]")?;
    let content = args.next().map(PathBuf::from);
    let frames = args
        .next()
        .and_then(|s| s.to_string_lossy().parse::<usize>().ok())
        .unwrap_or(1);
    let cfg = env::var_os("RETROFRONT_CONFIG")
        .map(PathBuf::from)
        .map_or_else(
            || Ok(FrontendConfig::default()),
            |p| FrontendConfig::load_retroarch_cfg(&p),
        )?;
    let mut host = RetroHost::load_core(&core, cfg)?;
    let meta = host.metadata();
    println!(
        "core={} version={} api={} extensions={}",
        meta.name,
        meta.version,
        host.api_version(),
        meta.valid_extensions
    );
    host.load_game(content.as_deref())?;
    apply_button_script(&host)?;
    for index in 0..frames {
        let frame = host.run_frame()?;
        println!(
            "frame={} video={}x{} pitch={} audio_frames={}",
            index + 1,
            frame.width,
            frame.height,
            frame.pitch,
            frame.audio_frames
        );
    }
    Ok(())
}

fn apply_button_script(host: &RetroHost) -> Result<(), String> {
    let Some(script) = env::var_os("RETROFRONT_BUTTONS") else {
        return Ok(());
    };
    for token in script
        .to_string_lossy()
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        let (button, value) = token
            .split_once('=')
            .ok_or_else(|| format!("invalid RETROFRONT_BUTTONS token {token}"))?;
        let id = parse_retro_id(button)?;
        let pressed = matches!(value, "1" | "true" | "pressed" | "down");
        host.set_joypad_button(0, id, pressed)?;
    }
    Ok(())
}
