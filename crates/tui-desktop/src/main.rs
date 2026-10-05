//! Escritorio TUI de TUI-OS. Salir: Q o Ctrl+Q.

mod app;
mod sys;
mod theme;
mod ui;

use std::time::{Duration, Instant};

use tui_core::Buffer;
use tui_render::{Renderer, Terminal};

/// `tui-desktop --snapshot [N]x[M] [sección]`: vuelca un fotograma como texto (sin terminal).
fn snapshot(spec: &str, section: Option<&str>) {
    let (w, h) = spec
        .split_once('x')
        .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
        .unwrap_or((120, 40));
    let mut app = app::App::new((w, h));
    app.tick();
    if let Some(i) = section.and_then(|s| s.parse::<usize>().ok()) {
        app.on_event(tui_core::Event::Key(tui_core::KeyEvent::new(
            tui_core::KeyCode::Char(char::from_digit(i as u32, 10).unwrap_or('0')),
            tui_core::Modifiers::NONE,
        )));
    }
    let mut buf = Buffer::new(w, h);
    app.draw(&mut buf);
    for y in 0..h {
        println!(
            "{}",
            (0..w)
                .map(|x| buf.get(x, y).map_or(' ', |c| c.ch))
                .collect::<String>()
                .trim_end()
        );
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--snapshot") {
        snapshot(
            args.get(1).map_or("120x40", String::as_str),
            args.get(2).map(String::as_str),
        );
        return Ok(());
    }
    let mut term = Terminal::enter()?;
    let mut renderer = if Terminal::is_linux_console() {
        Renderer::with_palette(theme::palette())
    } else {
        Renderer::new()
    };
    let (w, h) = term.size()?;
    let mut app = app::App::new((w, h));
    app.tick();
    let (mut w, mut h) = (w, h);
    let mut last_tick = Instant::now();
    while !app.quit {
        let mut buf = Buffer::new(w, h);
        app.draw(&mut buf);
        renderer.render(term.writer(), &buf)?;
        if let Some(ev) = term.poll_event(Duration::from_millis(200))? {
            if let tui_core::Event::Resize { width, height } = ev {
                (w, h) = (width, height);
            }
            app.on_event(ev);
        }
        if last_tick.elapsed() >= Duration::from_secs(1) {
            app.tick();
            last_tick = Instant::now();
        }
    }
    Ok(())
}
