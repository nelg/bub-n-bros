//! bubnbros-client: a native, single-binary client for The Bub's Brothers.
//!
//! It speaks the same protocol as display/Client.py and the Java applet,
//! so it can join any Bub-n-Bros server (Python 2 or 3).

mod audio;
mod discovery;
mod files;
mod game;
mod proto;

use game::{Game, Options, UdpMode};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() -> ! {
    eprintln!(
        "bubnbros-client {VERSION} -- native client for The Bub's Brothers

usage: bubnbros-client [options] [host[:port]]

  host:port          connect to the game server at this address
  host               look for a game server on this machine
  (nothing)          look for a game server on this machine and the LAN

options:
  -f, --fullscreen   start in full screen (F11 or Alt+Enter switch at any
                     time, Esc leaves full screen)
                     In the game, F7 / F8 turn the volume down / up.
  -z, --zoom=N       window size in percent of the game size (default 100)
  -t, --tcp          receive the game frames over TCP (slow or proxied links)
  -u, --udp          always use UDP (default: UDP, falling back to TCP)
  --no-sound         no sound at all
  --no-music         sound effects but no background music
  --volume=N         sound volume in percent (default 100, up to 300)
  --music-volume=N   music volume in percent of the sound volume (default 100)
  --name=NAME        name of your players
  --player=N=NAME    name of player number N (0-9)
  --cache=DIR        where to keep the game data downloaded from servers
  --no-cache         do not keep the downloaded game data
  -h, --help         show this help

In the game: click on one of the dragons at the bottom of the window,
then press the keys you want to use for right, left, jump and fire.
Click on your dragon again to leave the game."
    );
    std::process::exit(2)
}

struct Args {
    address: Option<String>,
    fullscreen: bool,
    zoom: f32,
    opts: Options,
    cachedir: Option<PathBuf>,
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
            "-h" | "--help" => usage(),
            "-V" | "--version" => {
                println!("bubnbros-client {VERSION}");
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
fn viewport(gw: f32, gh: f32, fullscreen: bool, zoom: f32) -> Rect {
    let (sw, sh) = (screen_width(), screen_height());
    let scale = if fullscreen || sw < gw * zoom - 1.0 || sh < gh * zoom - 1.0 {
        (sw / gw).min(sh / gh)
    } else {
        zoom
    };
    let (w, h) = (gw * scale, gh * scale);
    Rect::new(((sw - w) / 2.0).floor(), ((sh - h) / 2.0).floor(), w, h)
}

async fn run(mut game: Game, mut fullscreen: bool, zoom: f32) {
    let mut target: Option<(RenderTarget, f32, f32)> = None;
    let mut requested_size = (0.0, 0.0);
    let mut swallowed: Vec<KeyCode> = Vec::new();
    let mut last_mouse = (-1.0, -1.0);
    let verbose = std::env::var_os("BUBNBROS_VERBOSE").is_some();
    let mut next_report = 10.0;
    let mut volume_shown_until = 0.0;

    loop {
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
            if key == KeyCode::F7 || key == KeyCode::F8 {
                // volume down / up, in steps of 10%
                let step = if key == KeyCode::F8 { 0.1 } else { -0.1 };
                let v = ((game.volume() + step) * 10.0).round() / 10.0;
                game.set_volume(v);
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
        let vp = viewport(gw, gh, fullscreen, zoom);
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
        if time < volume_shown_until {
            let text = if game.has_sound() {
                format!("Volume {:.0}%", game.volume() * 100.0)
            } else {
                "Sound is off (--no-sound)".to_string()
            };
            draw_rectangle(10.0, 10.0, 260.0, 40.0, Color::from_rgba(0, 0, 0, 180));
            draw_text(&text, 20.0, 38.0, 30.0, WHITE);
        }
        next_frame().await;
    }
}

fn main() {
    let args = parse_args();
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
    let conf = Conf {
        window_title: format!("Bub-n-Bros - {}", game.gameident),
        window_width: 640,
        window_height: 480,
        fullscreen: args.fullscreen,
        window_resizable: true,
        high_dpi: false,
        ..Default::default()
    };
    let (fullscreen, zoom) = (args.fullscreen, args.zoom);
    macroquad::Window::from_config(conf, run(game, fullscreen, zoom));
}
