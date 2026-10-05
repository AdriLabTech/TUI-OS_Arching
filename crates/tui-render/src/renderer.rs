use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{
    Attribute, Color as CtColor, Print, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{Clear, ClearType};
use tui_core::{Attrs, Buffer, Color, Style};

/// Paleta de 16 colores para terminales sin truecolor (consola de Linux).
/// Los colores `Rgb` se asignan al índice más cercano; los índices 0-7 sirven también de fondo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette(pub [(u8, u8, u8); 16]);

impl Palette {
    pub fn nearest(&self, r: u8, g: u8, b: u8) -> u8 {
        let dist = |(pr, pg, pb): (u8, u8, u8)| {
            let d = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
            d(pr, r) + d(pg, g) + d(pb, b)
        };
        (0..16u8)
            .min_by_key(|&i| dist(self.0[i as usize]))
            .unwrap_or(0)
    }
}

/// Recuerda el último fotograma para emitir solo las diferencias.
#[derive(Default)]
pub struct Renderer {
    prev: Option<Buffer>,
    palette: Option<Palette>,
}

fn color(c: Color, palette: Option<&Palette>) -> CtColor {
    match (c, palette) {
        (Color::Reset, _) => CtColor::Reset,
        (Color::Indexed(i), _) => CtColor::AnsiValue(i),
        (Color::Rgb(r, g, b), Some(p)) => CtColor::AnsiValue(p.nearest(r, g, b)),
        (Color::Rgb(r, g, b), None) => CtColor::Rgb { r, g, b },
    }
}

fn queue_style<W: Write>(out: &mut W, s: Style, palette: Option<&Palette>) -> io::Result<()> {
    queue!(out, SetAttribute(Attribute::Reset))?;
    for (flag, attr) in [
        (Attrs::BOLD, Attribute::Bold),
        (Attrs::DIM, Attribute::Dim),
        (Attrs::ITALIC, Attribute::Italic),
        (Attrs::UNDERLINE, Attribute::Underlined),
        (Attrs::REVERSE, Attribute::Reverse),
    ] {
        // En la consola de Linux, negrita/tenue cambian el índice de color (brillo): se omiten.
        let changes_palette = matches!(attr, Attribute::Bold | Attribute::Dim);
        if s.attrs.contains(flag) && !(palette.is_some() && changes_palette) {
            queue!(out, SetAttribute(attr))?;
        }
    }
    queue!(
        out,
        SetForegroundColor(color(s.fg, palette)),
        SetBackgroundColor(color(s.bg, palette))
    )
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer::default()
    }

    /// Renderer para terminales de 16 colores: reprograma la paleta (OSC P) en cada repintado completo.
    pub fn with_palette(palette: Palette) -> Renderer {
        Renderer {
            prev: None,
            palette: Some(palette),
        }
    }

    /// Fuerza repintar todo en la próxima llamada (p. ej. tras salir de otro programa).
    pub fn invalidate(&mut self) {
        self.prev = None;
    }

    /// Pinta `buf`. El primer fotograma y cualquier cambio de tamaño repintan todo.
    pub fn render<W: Write>(&mut self, out: &mut W, buf: &Buffer) -> io::Result<()> {
        let full = self
            .prev
            .as_ref()
            .is_none_or(|p| (p.width(), p.height()) != (buf.width(), buf.height()));
        if full {
            if let Some(p) = &self.palette {
                for (i, (r, g, b)) in p.0.iter().enumerate() {
                    queue!(out, Print(format!("\x1b]P{i:X}{r:02x}{g:02x}{b:02x}")))?;
                }
            }
            queue!(out, SetAttribute(Attribute::Reset), Clear(ClearType::All))?;
        }
        let changed = match (&self.prev, full) {
            (Some(p), false) => buf.diff(p),
            _ => buf.diff(&Buffer::new(0, 0)),
        };
        let mut cursor: Option<(u16, u16)> = None;
        let mut current: Option<Style> = None;
        for (x, y, cell) in changed {
            if cell.is_continuation() {
                continue; // la dibuja el carácter ancho de su izquierda
            }
            if cursor != Some((x, y)) {
                queue!(out, MoveTo(x, y))?;
            }
            if current != Some(cell.style) {
                queue_style(out, cell.style, self.palette.as_ref())?;
                current = Some(cell.style);
            }
            queue!(out, Print(cell.ch))?;
            // En la última columna el cursor queda en un estado de "wrap pendiente": no lo damos por conocido.
            let next = x + cell.width();
            cursor = (next < buf.width()).then_some((next, y));
        }
        queue!(out, SetAttribute(Attribute::Reset))?;
        out.flush()?;
        self.prev = Some(buf.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_core::Cell;

    fn render(r: &mut Renderer, b: &Buffer) -> String {
        let mut out = Vec::new();
        r.render(&mut out, b).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn first_frame_clears_and_paints_everything() {
        let mut b = Buffer::new(3, 1);
        b.put_str(0, 0, "abc", Style::default());
        let s = render(&mut Renderer::new(), &b);
        assert!(s.contains("\x1b[2J"), "borra la pantalla: {s:?}");
        assert!(
            s.contains("abc"),
            "celdas contiguas sin MoveTo entre medias: {s:?}"
        );
    }

    #[test]
    fn unchanged_frame_emits_no_cell_output() {
        let mut r = Renderer::new();
        let b = Buffer::new(4, 2);
        render(&mut r, &b);
        let s = render(&mut r, &b);
        assert_eq!(s, "\x1b[0m", "solo el reset de atributos");
    }

    #[test]
    fn single_cell_change_moves_and_prints_only_it() {
        let mut r = Renderer::new();
        let mut b = Buffer::new(10, 5);
        render(&mut r, &b);
        b.set(7, 3, Cell::new('Z', Style::default()));
        let s = render(&mut r, &b);
        assert!(
            s.contains("\x1b[4;8H"),
            "MoveTo(7,3) = fila 4, col 8 (1-based): {s:?}"
        );
        assert_eq!(s.matches('Z').count(), 1);
        assert!(!s.contains("\x1b[2J"));
    }

    #[test]
    fn resize_repaints_everything() {
        let mut r = Renderer::new();
        render(&mut r, &Buffer::new(2, 2));
        let s = render(&mut r, &Buffer::new(3, 2));
        assert!(s.contains("\x1b[2J"));
        assert_eq!(s.matches(' ').count(), 6);
    }

    #[test]
    fn style_is_emitted_only_when_it_changes() {
        let bold = Style::new(Color::Indexed(1), Color::Reset, Attrs::BOLD);
        let mut b = Buffer::new(4, 1);
        b.put_str(0, 0, "aaaa", bold);
        let s = render(&mut Renderer::new(), &b);
        assert_eq!(s.matches("\x1b[1m").count(), 1, "{s:?}");
        assert!(s.contains("\x1b[38;5;1m"), "{s:?}");
    }

    fn pal() -> Palette {
        let mut c = [(0, 0, 0); 16];
        c[1] = (226, 52, 42);
        c[6] = (205, 196, 186);
        c[9] = (149, 143, 135);
        Palette(c)
    }

    #[test]
    fn palette_is_programmed_on_full_repaint_only() {
        let mut r = Renderer::with_palette(pal());
        let b = Buffer::new(2, 1);
        let s = render(&mut r, &b);
        assert_eq!(s.matches("\x1b]P").count(), 16);
        assert!(s.contains("\x1b]P6cdc4ba"), "{s:?}");
        assert!(!render(&mut r, &b).contains("\x1b]P"));
    }

    #[test]
    fn rgb_maps_to_nearest_palette_slot() {
        assert_eq!(pal().nearest(205, 196, 186), 6);
        assert_eq!(pal().nearest(220, 60, 50), 1);
        assert_eq!(pal().nearest(150, 140, 130), 9);
        let mut b = Buffer::new(1, 1);
        b.set(
            0,
            0,
            Cell::new(
                'x',
                Style::new(Color::Rgb(226, 52, 42), Color::Rgb(0, 0, 0), Attrs::NONE),
            ),
        );
        let s = render(&mut Renderer::with_palette(pal()), &b);
        assert!(s.contains("\x1b[38;5;1m") && !s.contains("38;2;"), "{s:?}");
    }

    #[test]
    fn palette_mode_drops_bold_but_truecolor_keeps_it() {
        let mut b = Buffer::new(1, 1);
        b.set(
            0,
            0,
            Cell::new('x', Style::new(Color::Reset, Color::Reset, Attrs::BOLD)),
        );
        assert!(!render(&mut Renderer::with_palette(pal()), &b).contains("\x1b[1m"));
        assert!(render(&mut Renderer::new(), &b).contains("\x1b[1m"));
    }

    #[test]
    fn truecolor_renderer_never_emits_palette_sequences() {
        let s = render(&mut Renderer::new(), &Buffer::new(2, 1));
        assert!(!s.contains("\x1b]P"));
    }

    #[test]
    fn invalidate_forces_full_repaint() {
        let mut r = Renderer::new();
        let b = Buffer::new(2, 2);
        render(&mut r, &b);
        r.invalidate();
        assert!(render(&mut r, &b).contains("\x1b[2J"));
    }
}

#[cfg(test)]
mod wide_tests {
    use super::*;
    use tui_core::Style;

    #[test]
    fn wide_char_is_printed_once_and_continuation_is_skipped() {
        let mut r = Renderer::new();
        let mut b = Buffer::new(6, 1);
        b.put_str(0, 0, "日x", Style::default());
        let mut out = Vec::new();
        r.render(&mut out, &b).unwrap();
        let s = String::from_utf8_lossy(&out);
        assert_eq!(s.matches('日').count(), 1);
        assert!(!s.contains('\0'));
        // 'x' va justo tras el ancho 2: el cursor no necesita reposicionarse.
        let tail = s.split('日').nth(1).unwrap();
        assert!(tail.trim_start_matches(|c| c != 'x').starts_with('x'));
        assert!(!tail.split('x').next().unwrap().contains("\x1b[1;"));
    }
}
