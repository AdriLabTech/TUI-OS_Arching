//! Terminal embebido de TUI-OS (M5): un programa real en un PTY, interpretado por `vt100`
//! y volcado a una `Surface` del compositor.
//!
//! Un hilo lee el PTY y manda los bytes por un canal; el hilo de la UI los consume con
//! [`Terminal::poll`], sin bloquearse.

mod compat;
mod keys;

use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use tui_core::{Attrs, Cell, Color, KeyEvent, Style, Surface};

pub use portable_pty::CommandBuilder as Command;

pub struct Terminal {
    parser: vt100::Parser,
    compat: compat::Compat,
    rx: Receiver<Vec<u8>>,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    /// El lector llegó a EOF: el hijo cerró el PTY.
    closed: bool,
}

impl Terminal {
    /// Lanza `cmd` en un PTY de `cols` × `rows`.
    pub fn spawn(cmd: CommandBuilder, cols: u16, rows: u16) -> Result<Terminal, String> {
        let (cols, rows) = (cols.max(1), rows.max(1));
        let pair = native_pty_system()
            .openpty(size(cols, rows))
            .map_err(|e| format!("openpty: {e}"))?;
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("spawn: {e}"))?;
        // El extremo esclavo debe cerrarse aquí o el lector no vería EOF al salir el hijo.
        drop(pair.slave);
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("reader: {e}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("writer: {e}"))?;

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            // EOF o EIO (Linux, al cerrarse el esclavo) terminan el hilo; al soltarse `tx`
            // el receptor lo ve como desconexión.
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
        });

        Ok(Terminal {
            parser: vt100::Parser::new(rows, cols, 0),
            compat: compat::Compat::default(),
            rx,
            writer,
            master: pair.master,
            child,
            closed: false,
        })
    }

    /// Consume la salida pendiente del hijo. Devuelve si la pantalla puede haber cambiado.
    pub fn poll(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.rx.try_recv() {
                Ok(bytes) => {
                    self.parser.process(&self.compat.filter(&bytes));
                    changed = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.closed = true;
                    break;
                }
            }
        }
        changed
    }

    /// ¿Sigue vivo el hijo? Falso cuando ha salido.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// El hijo cerró el PTY y ya se consumió toda su salida.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn send_key(&mut self, key: KeyEvent) -> std::io::Result<()> {
        let bytes = keys::encode(key, self.parser.screen().application_cursor());
        self.send_bytes(&bytes)
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    /// Cambia el tamaño del PTY (el hijo recibe SIGWINCH) y de la pantalla interpretada.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let (cols, rows) = (cols.max(1), rows.max(1));
        if self.parser.screen().size() == (rows, cols) {
            return;
        }
        let _ = self.master.resize(size(cols, rows));
        self.parser.screen_mut().set_size(rows, cols);
    }

    /// Texto de la pantalla, para pruebas y diagnóstico.
    pub fn contents(&self) -> String {
        self.parser.screen().contents()
    }

    /// Vuelca la pantalla en `surface` (a su tamaño) y devuelve la posición del cursor
    /// `(x, y)` si es visible.
    pub fn draw(&self, surface: &mut Surface) -> Option<(u16, u16)> {
        let (w, h) = (surface.width(), surface.height());
        self.draw_with(w, h, |x, y, cell| surface.set(x, y, cell))
    }

    /// Como [`draw`](Terminal::draw), pero entrega cada celda a `put` (recortada a
    /// `width` × `height`), para dibujar en cualquier destino.
    pub fn draw_with(
        &self,
        width: u16,
        height: u16,
        mut put: impl FnMut(u16, u16, Cell),
    ) -> Option<(u16, u16)> {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        for y in 0..rows.min(height) {
            for x in 0..cols.min(width) {
                let cell = match screen.cell(y, x) {
                    Some(c) if c.is_wide_continuation() => Cell::continuation(style_of(c)),
                    Some(c) => {
                        // M5-02: un grafema entero no cabe en `char`; de momento el primero.
                        let ch = c.contents().chars().next().unwrap_or(' ');
                        Cell::new(ch, style_of(c))
                    }
                    None => Cell::default(),
                };
                put(x, y, cell);
            }
        }
        let (cy, cx) = screen.cursor_position();
        (!screen.hide_cursor()).then_some((cx, cy))
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Sin esto el hijo seguiría vivo huérfano al cerrar la ventana.
        let _ = self.child.kill();
    }
}

fn size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn color_of(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn style_of(c: &vt100::Cell) -> Style {
    let mut attrs = Attrs::NONE;
    for (on, a) in [
        (c.bold(), Attrs::BOLD),
        (c.dim(), Attrs::DIM),
        (c.italic(), Attrs::ITALIC),
        (c.underline(), Attrs::UNDERLINE),
        (c.inverse(), Attrs::REVERSE),
    ] {
        if on {
            attrs = attrs.union(a);
        }
    }
    Style::new(color_of(c.fgcolor()), color_of(c.bgcolor()), attrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use tui_core::{KeyCode, Modifiers};

    fn sh(script: &str) -> CommandBuilder {
        let mut cmd = CommandBuilder::new("sh");
        cmd.args(["-c", script]);
        cmd
    }

    /// Hace `poll` hasta que la pantalla contiene `needle` o vence el plazo.
    fn wait_for(term: &mut Terminal, needle: &str) -> bool {
        let end = Instant::now() + Duration::from_secs(5);
        while Instant::now() < end {
            term.poll();
            if term.contents().contains(needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[test]
    fn ejecuta_un_programa_y_muestra_su_salida() {
        let mut t = Terminal::spawn(sh("echo hola-tui"), 40, 5).unwrap();
        assert!(wait_for(&mut t, "hola-tui"), "pantalla: {:?}", t.contents());
    }

    #[test]
    fn el_hijo_ve_el_tamano_y_el_resize() {
        let mut t = Terminal::spawn(sh("stty size; read x; stty size"), 40, 5).unwrap();
        assert!(wait_for(&mut t, "5 40"), "pantalla: {:?}", t.contents());
        t.resize(60, 10);
        t.send_bytes(b"\n").unwrap();
        assert!(wait_for(&mut t, "10 60"), "pantalla: {:?}", t.contents());
    }

    #[test]
    fn el_teclado_llega_al_hijo() {
        let mut t = Terminal::spawn(sh("read x; echo recibido-$x"), 40, 5).unwrap();
        for c in "ab".chars() {
            t.send_key(KeyEvent::new(KeyCode::Char(c), Modifiers::NONE))
                .unwrap();
        }
        t.send_key(KeyEvent::new(KeyCode::Enter, Modifiers::NONE))
            .unwrap();
        assert!(
            wait_for(&mut t, "recibido-ab"),
            "pantalla: {:?}",
            t.contents()
        );
    }

    #[test]
    fn detecta_que_el_hijo_termino() {
        let mut t = Terminal::spawn(sh("exit 0"), 20, 3).unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        while !t.is_closed() && Instant::now() < end {
            t.poll();
            thread::sleep(Duration::from_millis(10));
        }
        assert!(t.is_closed());
        let end = Instant::now() + Duration::from_secs(5);
        while t.is_running() && Instant::now() < end {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(!t.is_running());
    }

    #[test]
    fn posiciona_el_cursor_con_hvp_como_btop() {
        // `CSI 2;3 f` (HVP) no lo implementa vt100: lo traduce `compat`.
        let mut t = Terminal::spawn(sh(r"printf '\033[2;3fX'"), 20, 4).unwrap();
        assert!(wait_for(&mut t, "X"));
        let mut s = Surface::new(20, 4);
        t.draw(&mut s);
        assert_eq!(s.buffer().get(2, 1).unwrap().ch, 'X');
    }

    #[test]
    fn dibuja_colores_y_atributos_en_la_superficie() {
        let mut t = Terminal::spawn(sh(r"printf '\033[1;31mX\033[0m ñ'"), 20, 3).unwrap();
        assert!(wait_for(&mut t, "ñ"));
        let mut s = Surface::new(20, 3);
        t.draw(&mut s);
        let x = s.buffer().get(0, 0).unwrap();
        assert_eq!(x.ch, 'X');
        assert_eq!(x.style.fg, Color::Indexed(1));
        assert!(x.style.attrs.contains(Attrs::BOLD));
        assert_eq!(s.buffer().get(2, 0).unwrap().ch, 'ñ');
    }
}
