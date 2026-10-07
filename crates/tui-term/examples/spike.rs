//! Spike de M5-01: ejecuta un programa en un `Terminal`, vuelca la pantalla y la
//! vuelve a volcar tras un resize. Uso: `cargo run -p tui-term --example spike -- top`.

use std::thread::sleep;
use std::time::Duration;
use tui_core::Surface;
use tui_term::{Command, Terminal};

fn dump(t: &mut Terminal, w: u16, h: u16) {
    sleep(Duration::from_millis(1500));
    t.poll();
    let mut s = Surface::new(w, h);
    t.draw(&mut s);
    let mut colored = 0;
    for y in 0..h {
        let line: String = (0..w)
            .map(|x| {
                let c = s.buffer().get(x, y).unwrap();
                if c.style.fg != tui_core::Color::Reset || c.style.bg != tui_core::Color::Reset {
                    colored += 1;
                }
                if c.is_continuation() { ' ' } else { c.ch }
            })
            .collect();
        println!("|{}", line.trim_end());
    }
    println!(
        "-- {w}x{h}, celdas con color: {colored}, vivo: {}",
        t.is_running()
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd.env("TERM", "xterm-256color");
    let mut t = Terminal::spawn(cmd, 100, 30).expect("spawn");
    dump(&mut t, 100, 30);
    t.resize(70, 20);
    dump(&mut t, 70, 20);
}
