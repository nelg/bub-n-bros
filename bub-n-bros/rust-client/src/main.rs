//! bubbros-client: a single-binary client, in Rust, for The Bub's Brothers.
//!
//! It speaks the same protocol as display/Client.py and the Java applet,
//! so it can join any Bub-n-Bros server (Python 2 or 3).

mod audio;
mod discovery;
mod files;
mod game;
mod proto;
mod menu;
mod single;
mod solo;

use game::{Game, Options, UdpMode};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Wrong arguments: the help on stderr, exit code 2.
fn usage() -> ! {
    eprintln!("{}", help_text());
    std::process::exit(2)
}

fn help_text() -> String {
    format!(
        "bubbros-client {VERSION} -- Rust client for The Bub's Brothers

usage: bubbros-client [options] [host[:port]]

  host:port          connect to the game server at this address
  host               look for a game server on this machine
  (nothing)          look for a game server on this machine; if there is
                     none, play alone: start a game server just for you

options:
  -f, --fullscreen   start in full screen (F11 or Alt+Enter switch at any
                     time)
  -z, --zoom=N       initial window size in percent of the game size
                     (default 100); the game always fills the window
  -t, --tcp          receive the game frames over TCP (slow or proxied links)
  -u, --udp          always use UDP (default: UDP, falling back to TCP)
  --no-sound         no sound at all
  --no-music         sound effects but no background music
  --volume=N         sound volume in percent (default 100, up to 200)
  --music-volume=N   music volume in percent of the sound volume (default 100)
  --name=NAME        name of your players
  --player=N=NAME    name of player number N (0-9)
  --cache=DIR        where to keep the game data downloaded from servers
  --no-cache         do not keep the downloaded game data
  --solo             play alone: start a game server just for this client
  --level=NAME       play alone with this level set (Levels, LostLevels,
                     HouseOfFun, Arena, CompactLevels, RandomLevels,
                     scratch) instead of choosing it on the screen
  --unlimited-lives  play alone with unlimited lives (default: 5 lives,
                     one more every 50000 points)
  --single-instance  one client per screen (used by the web launcher): exit
                     if another started less than 30 s ago, otherwise
                     stop it and take its place
  -h, --help         show this help

In the game: click on one of the dragons at the bottom of the window,
then press the keys you want to use for right, left, jump and fire.
Click on your dragon again to leave the game.

  F11, Alt+Enter     full screen on / off
  F7 / F8            volume down / up
  F9                 mute / unmute
  Esc                leave full screen, or quit"
    )
}

struct Args {
    address: Option<String>,
    fullscreen: bool,
    zoom: f32,
    opts: Options,
    cachedir: Option<PathBuf>,
    single_instance: bool,
    solo: bool,
    no_arguments: bool,
    level: Option<String>,
    unlimited_lives: bool,
}

fn parse_args() -> Args {
    let mut a = Args {
        address: None,
        fullscreen: false,
        zoom: 1.0,
        opts: Options {
            udp_mode: UdpMode::Auto,
            sound: true,
            music: true,
            names: HashMap::new(),
            default_name: None,
            volume: 1.0,
            music_volume: 1.0,
        },
        cachedir: files::default_cache_dir(),
        single_instance: false,
        solo: false,
        no_arguments: std::env::args().len() <= 1,
        level: None,
        unlimited_lives: false,
    };
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        let (key, inline) = match arg.split_once('=') {
            Some((k, v)) if k.starts_with("--") => (k.to_string(), Some(v.to_string())),
            _ => (arg.clone(), None),
        };
        let value = |args: &mut std::iter::Peekable<std::iter::Skip<std::env::Args>>| {
            inline.clone().or_else(|| args.next()).unwrap_or_else(|| usage())
        };
        match key.as_str() {
            "-f" | "--fullscreen" => {
                a.fullscreen = inline.as_deref().map_or(true, |v| !v.starts_with('n'))
            }
            "-z" | "--zoom" => {
                let v = value(&mut args);
                a.zoom = v.trim_end_matches('%').parse::<f32>().unwrap_or_else(|_| usage()) / 100.0;
            }
            "-t" | "--tcp" => a.opts.udp_mode = UdpMode::Tcp,
            "-u" | "--udp" => a.opts.udp_mode = UdpMode::Udp,
            "--no-sound" => a.opts.sound = false,
            "--no-music" => a.opts.music = false,
            "--volume" | "--music-volume" => {
                let v = value(&mut args);
                let pc = v.trim_end_matches('%').parse::<f32>().unwrap_or_else(|_| usage());
                if key == "--volume" {
                    a.opts.volume = (pc / 100.0).clamp(0.0, audio::MAX_VOLUME);
                } else {
                    a.opts.music_volume = (pc / 100.0).clamp(0.0, audio::MAX_VOLUME);
                }
            }
            "--name" => a.opts.default_name = Some(value(&mut args)),
            "--player" => {
                let v = value(&mut args);
                let (n, name) = v.split_once('=').unwrap_or_else(|| usage());
                a.opts.names.insert(n.parse().unwrap_or_else(|_| usage()), name.to_string());
            }
            "--cache" => a.cachedir = Some(PathBuf::from(value(&mut args))),
            "--no-cache" => a.cachedir = None,
            "--single-instance" => a.single_instance = true,
            "--solo" => a.solo = true,
            "--level" => {
                let v = value(&mut args);
                a.level = Some(solo::level_file(&v).unwrap_or_else(|| {
                    eprintln!("unknown level set {:?}", v);
                    usage()
                }));
                a.solo = true;
            }
            "--unlimited-lives" => a.unlimited_lives = true,
            "-h" | "--help" => {
                println!("{}", help_text());
                std::process::exit(0)
            }
            "-V" | "--version" => {
                println!("bubbros-client {VERSION}");
                std::process::exit(0)
            }
            s if s.starts_with('-') => {
                eprintln!("unknown option {}", s);
                usage()
            }
            _ => {
                if a.address.is_some() {
                    usage()
                }
                a.address = Some(arg)
            }
        }
    }
    a
}

fn find_server(address: Option<&str>) -> SocketAddr {
    let found = match address {
        Some(a) if a.contains(':') => {
            match a.to_socket_addrs().ok().and_then(|mut it| it.find(|x| x.is_ipv4()).or_else(|| a.to_socket_addrs().ok()?.next())) {
                Some(addr) => return addr,
                None => {
                    eprintln!("cannot resolve {}", a);
                    std::process::exit(1)
                }
            }
        }
        Some(host) => discovery::pick(&[host], 5),
        None => discovery::pick(&["127.0.0.1", "<broadcast>"], 3),
    };
    match found {
        Some((addr, _)) => addr,
        None => {
            eprintln!("no server found.");
            std::process::exit(1)
        }
    }
}

/// Where the game image goes in the window: (x, y, width, height).
/// The game is scaled to fill the window (or the screen), keeping its
/// proportions; --zoom only chooses the initial window size.
fn viewport(gw: f32, gh: f32) -> Rect {
    let (sw, sh) = (screen_width(), screen_height());
    let scale = (sw / gw).min(sh / gh);
    let (w, h) = (gw * scale, gh * scale);
    Rect::new(((sw - w) / 2.0).floor(), ((sh - h) / 2.0).floor(), w, h)
}

/// The client's own keys, shown until somebody joins the game.
fn draw_help(vp: Rect, solo: bool) {
    const SOLO: &str = "Play alone mode: no server was found, so one runs just for you";
    const KEYS: [&str; 2] = ["F11: full screen    Esc: leave full screen / quit",
                             "F7 / F8: volume down / up    F9: mute"];
    let mut lines: Vec<&str> = Vec::new();
    if solo {
        lines.push(SOLO);
    }
    lines.extend_from_slice(&KEYS);
    let size = (vp.h / 25.0).clamp(12.0, 32.0);
    let width = lines.iter()
        .map(|l| measure_text(l, None, size as u16, 1.0).width)
        .fold(0.0, f32::max);
    let (x, y) = (vp.x + (vp.w - width) / 2.0, vp.y + size * 0.4);
    draw_rectangle(x - size * 0.5, y, width + size, size * (0.4 + 1.1 * lines.len() as f32),
                   Color::from_rgba(0, 0, 0, 170));
    for (i, line) in lines.iter().enumerate() {
        let color = if solo && i == 0 {
            Color::from_rgba(140, 255, 140, 255)
        } else {
            Color::from_rgba(255, 255, 160, 255)
        };
        draw_text(line, x, y + size * (1.0 + 1.1 * i as f32), size, color);
    }
}

async fn run(mut game: Game, mut fullscreen: bool, zoom: f32, solo: bool) {
    let mut target: Option<(RenderTarget, f32, f32)> = None;
    let mut requested_size = (0.0, 0.0);
    let mut swallowed: Vec<KeyCode> = Vec::new();
    let mut last_mouse = (-1.0, -1.0);
    let verbose = std::env::var_os("BUBNBROS_VERBOSE").is_some();
    let mut next_report = 10.0;
    let mut volume_shown_until = 0.0;
    // closing the window: stop the play-alone server before leaving
    prevent_quit();

    loop {
        if is_quit_requested() {
            break;
        }
        let time = get_time();
        game.poll(time);
        if let Some(reason) = &game.closed {
            eprintln!("{}", reason);
            break;
        }
        let (gw, gh) = (game.width, game.height);
        if verbose && time >= next_report {
            eprintln!("[{:.0}s] {}", time, game.stats());
            next_report += 10.0;
        }

        if game.defined && !fullscreen && requested_size != (gw, gh) {
            request_new_screen_size(gw * zoom, gh * zoom);
            requested_size = (gw, gh);
        }

        // keyboard: F11 / Alt+Enter / Esc are for us, the rest for the game
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        for key in get_keys_pressed() {
            let toggle = key == KeyCode::F11
                || (key == KeyCode::Enter && alt)
                || (key == KeyCode::KpEnter && alt)
                || (key == KeyCode::Escape && fullscreen);
            if key == KeyCode::Escape && !fullscreen {
                // Esc leaves full screen, or else quits
                solo::stop();
                std::process::exit(0);
            }
            if key == KeyCode::F9 {
                game.set_muted(!game.muted());
                volume_shown_until = time + 2.0;
                swallowed.push(key);
                continue;
            }
            if key == KeyCode::F7 || key == KeyCode::F8 {
                // volume down / up, in steps of 20%
                let step = if key == KeyCode::F8 { 0.2 } else { -0.2 };
                let v = ((game.volume() + step) * 5.0).round() / 5.0;
                game.set_volume(v);
                game.set_muted(false);
                volume_shown_until = time + 2.0;
                swallowed.push(key);
                continue;
            }
            if toggle {
                fullscreen = !fullscreen;
                set_fullscreen(fullscreen);
                if !fullscreen {
                    requested_size = (0.0, 0.0);
                }
                swallowed.push(key);
            } else {
                game.key_event(key, true);
            }
        }
        for key in get_keys_released() {
            if let Some(i) = swallowed.iter().position(|&k| k == key) {
                swallowed.remove(i);
            } else {
                game.key_event(key, false);
            }
        }

        // mouse, mapped to game coordinates
        let vp = viewport(gw, gh);
        let (mx, my) = mouse_position();
        let gx = (mx - vp.x) * gw / vp.w;
        let gy = (my - vp.y) * gh / vp.h;
        if (mx, my) != last_mouse {
            last_mouse = (mx, my);
            if game.defined {
                game.pointer_motion(gx, gy, time);
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) && game.defined {
            game.mouse_click(gx, gy, time);
        }

        // draw the game at its native size into a texture, then scale it
        clear_background(BLACK);
        if game.defined {
            let rt = match &target {
                Some((rt, w, h)) if *w == gw && *h == gh => rt.clone(),
                _ => {
                    let rt = render_target(gw as u32, gh as u32);
                    rt.texture.set_filter(FilterMode::Linear);
                    target = Some((rt.clone(), gw, gh));
                    rt
                }
            };
            set_camera(&Camera2D {
                zoom: vec2(2.0 / gw, 2.0 / gh),
                target: vec2(gw / 2.0, gh / 2.0),
                render_target: Some(rt.clone()),
                ..Default::default()
            });
            game.draw(time);
            set_default_camera();
            draw_texture_ex(&rt.texture, vp.x, vp.y, WHITE, DrawTextureParams {
                dest_size: Some(vec2(vp.w, vp.h)),
                ..Default::default()
            });
        } else {
            draw_text(&format!("Connecting to {}...", game.gameident),
                      20.0, 40.0, 24.0, WHITE);
        }
        if game.defined && !game.has_local_player() {
            draw_help(vp, solo);
        }
        if time < volume_shown_until {
            let text = if !game.has_sound() {
                "Sound is off (--no-sound)".to_string()
            } else if game.muted() {
                "Sound muted (F9)".to_string()
            } else {
                format!("Volume {:.0}%", game.volume() * 100.0)
            };
            draw_rectangle(10.0, 10.0, 300.0, 40.0, Color::from_rgba(0, 0, 0, 180));
            draw_text(&text, 20.0, 38.0, 30.0, WHITE);
        }
        next_frame().await;
    }
    solo::stop();
    std::process::exit(0);
}

fn main() {
    let args = parse_args();
    // started by the web launcher: one client per screen
    let _lock = if args.single_instance {
        match single::acquire() {
            Some(lock) => lock,
            None => {
                eprintln!("A Bub-n-Bros client is already running on this screen \
                           (lock {}); not starting another one.",
                          single::lock_path().display());
                std::process::exit(0)
            }
        }
    } else {
        None
    };
    // play alone: when asked to (--solo), or when the client is started
    // without any argument and no server runs on this computer
    let mut solo = args.solo;
    if !solo && args.no_arguments {
        if discovery::pick(&["127.0.0.1"], 2).is_none() {
            eprintln!("No game server on this computer: playing alone.");
            solo = true;
        }
    }
    let conf = |title: String| Conf {
        window_title: title,
        window_width: 640,
        window_height: 480,
        fullscreen: args.fullscreen,
        window_resizable: true,
        high_dpi: false,
        ..Default::default()
    };
    let (fullscreen, zoom) = (args.fullscreen, args.zoom);

    if solo {
        // play alone: the window opens first, to choose what to play
        let cache = args.cachedir.clone().unwrap_or_else(|| std::env::temp_dir().join("bubnbros"));
        let preset = args.level.clone().map(|level| solo::Settings {
            level, unlimited_lives: args.unlimited_lives });
        let initial = solo::Settings {
            level: "Levels.bin".into(), unlimited_lives: args.unlimited_lives };
        let (opts, cachedir) = (args.opts, args.cachedir);
        macroquad::Window::from_config(conf("Bub-n-Bros - play alone".into()), async move {
            prevent_quit();
            let settings = match preset {
                Some(s) => s,
                None => match menu::choose(initial).await {
                    Some(s) => s,
                    None => std::process::exit(0),
                },
            };
            menu::message("Starting the game server...").await;
            menu::message("Starting the game server...").await;
            let addr = match solo::start(&cache, &settings) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("cannot play alone: {}", e);
                    menu::error(&format!("Cannot play alone:\n{}", e)).await;
                    std::process::exit(1)
                }
            };
            eprintln!("connecting to {}...", addr);
            let game = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
                .map_err(|e| e.to_string())
                .and_then(|tcp| Game::new(tcp, opts, cachedir).map_err(|e| e.to_string()));
            match game {
                Ok(game) => run(game, fullscreen, zoom, true).await,
                Err(e) => {
                    eprintln!("cannot connect to the game server: {}", e);
                    solo::stop();
                    menu::error(&format!("Cannot connect to the game server:\n{}", e)).await;
                    std::process::exit(1)
                }
            }
        });
        solo::stop();
        return;
    }

    let addr = find_server(args.address.as_deref());
    eprintln!("connecting to {}...", addr);
    let tcp = match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot connect to {}: {}", addr, e);
            std::process::exit(1)
        }
    };
    let game = match Game::new(tcp, args.opts, args.cachedir) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        }
    };
    macroquad::Window::from_config(conf(format!("Bub-n-Bros - {}", game.gameident)),
                                   run(game, fullscreen, zoom, false));
}
