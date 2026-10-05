//! Primitivas de dibujo sobre `Buffer`. Solo glifos presentes en la fuente de consola
//! Terminus (ter-v16n, comprobado en su tabla Unicode): `█ ░ ▒ ■`, líneas simples
//! `─│┌┐└┘├┤┬┴┼` y flechas `↑↓←→►◄`. NO existen `▀ ▄ ▌ ▐ ▓` ni líneas dobles.
//! Candidatas a promoverse a una crate de widgets cuando exista el compositor (M3).

use tui_core::{Buffer, Cell, Color, Rect, Style};

use crate::theme::{self, style};

pub fn inset(r: Rect, m: u16) -> Rect {
    Rect::new(
        r.x.saturating_add(m),
        r.y.saturating_add(m),
        r.width.saturating_sub(2 * m),
        r.height.saturating_sub(2 * m),
    )
}

pub fn fill(b: &mut Buffer, r: Rect, bg: Color) {
    let cell = Cell::new(' ', style(theme::INK, bg));
    for y in r.y..r.y.saturating_add(r.height) {
        for x in r.x..r.x.saturating_add(r.width) {
            b.set(x, y, cell);
        }
    }
}

/// Texto recortado a `max` columnas. Devuelve las columnas escritas.
pub fn text(b: &mut Buffer, x: u16, y: u16, max: u16, s: &str, st: Style) -> u16 {
    let clipped: String = s.chars().take(max as usize).collect();
    b.put_str(x, y, &clipped, st)
}

pub fn hline(b: &mut Buffer, x: u16, y: u16, w: u16, ch: char, st: Style) {
    for i in 0..w {
        b.set(x.saturating_add(i), y, Cell::new(ch, st));
    }
}

/// Marco de línea simple; el título va incrustado en el borde superior.
pub fn frame(b: &mut Buffer, r: Rect, title: &str, active: bool, bg: Color) {
    if r.width < 2 || r.height < 2 {
        return;
    }
    let line = style(if active { theme::ACTIVE } else { theme::LINE }, bg);
    let (x1, y1) = (r.x + r.width - 1, r.y + r.height - 1);
    hline(b, r.x, r.y, r.width, '─', line);
    hline(b, r.x, y1, r.width, '─', line);
    for y in r.y..=y1 {
        b.set(r.x, y, Cell::new('│', line));
        b.set(x1, y, Cell::new('│', line));
    }
    b.set(r.x, r.y, Cell::new('┌', line));
    b.set(x1, r.y, Cell::new('┐', line));
    b.set(r.x, y1, Cell::new('└', line));
    b.set(x1, y1, Cell::new('┘', line));
    if !title.is_empty() && r.width > 6 {
        let label = style(if active { theme::BRIGHT } else { theme::MUTED }, bg);
        text(b, r.x + 2, r.y, r.width - 4, &format!(" {title} "), label);
    }
}

/// Barra horizontal: `█` lleno y `░` vacío.
pub fn hbar(b: &mut Buffer, x: u16, y: u16, w: u16, frac: f32, fg: Color, bg: Color) {
    let filled = ((frac.clamp(0.0, 1.0) * w as f32).round() as u16).min(w);
    hline(b, x, y, filled, '█', style(fg, bg));
    hline(b, x + filled, y, w - filled, '░', style(theme::LINE, bg));
}

/// Gráfico de área con las últimas `r.width` muestras (0..=1), alineado a la derecha.
/// Filas completas `█` y, arriba, un `▒` (≥ media fila) o `░` (resto) para el parcial.
/// Con valor ≈ 0 deja una línea base tenue para que el gráfico siga visible.
pub fn area(b: &mut Buffer, r: Rect, data: &[f32], fg: Color, bg: Color) {
    if r.is_empty() {
        return;
    }
    let st = style(fg, bg);
    let w = r.width as usize;
    let skip = data.len().saturating_sub(w);
    let off = w.saturating_sub(data.len());
    for (i, v) in data.iter().skip(skip).enumerate() {
        let x = r.x + (off + i) as u16;
        let rows = v.clamp(0.0, 1.0) * r.height as f32;
        let full = rows.floor() as u16;
        let rem = rows - full as f32;
        for row in 0..full.min(r.height) {
            b.set(x, r.y + r.height - 1 - row, Cell::new('█', st));
        }
        if full < r.height {
            let y = r.y + r.height - 1 - full;
            match rem {
                m if m >= 0.5 => b.set(x, y, Cell::new('▒', st)),
                m if m > 0.1 => b.set(x, y, Cell::new('░', st)),
                _ if full == 0 => b.set(x, y, Cell::new('░', style(theme::LINE, bg))),
                _ => false,
            };
        }
    }
}

/// `ETIQUETA ···· valor`, estilo ficha técnica.
pub fn leader(b: &mut Buffer, x: u16, y: u16, w: u16, label: &str, value: &str, bg: Color) {
    let l = label.chars().count() as u16;
    let v = value.chars().count() as u16;
    text(b, x, y, w, label, style(theme::MUTED, bg));
    if w > l + v + 2 {
        hline(b, x + l + 1, y, w - l - v - 2, '·', style(theme::LINE, bg));
        text(b, x + w - v, y, v, value, style(theme::INK, bg));
    } else {
        text(
            b,
            x + l + 1,
            y,
            w.saturating_sub(l + 1),
            value,
            style(theme::INK, bg),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(b: &Buffer, y: u16) -> String {
        (0..b.width()).map(|x| b.get(x, y).unwrap().ch).collect()
    }

    #[test]
    fn frame_draws_corners_and_title() {
        let mut b = Buffer::new(12, 3);
        frame(&mut b, Rect::new(0, 0, 12, 3), "SYS", false, theme::PAPER);
        assert_eq!(row(&b, 0), "┌─ SYS ────┐");
        assert_eq!(row(&b, 2), "└──────────┘");
        assert_eq!(b.get(0, 1).unwrap().ch, '│');
    }

    #[test]
    fn frame_survives_tiny_and_offscreen_rects() {
        let mut b = Buffer::new(4, 2);
        frame(&mut b, Rect::new(0, 0, 1, 1), "x", true, theme::PAPER);
        frame(&mut b, Rect::new(3, 1, 50, 50), "x", true, theme::PAPER);
    }

    #[test]
    fn hbar_fills_proportionally() {
        let mut b = Buffer::new(10, 1);
        hbar(&mut b, 0, 0, 10, 0.5, theme::INK, theme::PAPER);
        assert_eq!(row(&b, 0), "█████░░░░░");
        hbar(&mut b, 0, 0, 10, 7.0, theme::INK, theme::PAPER);
        assert_eq!(row(&b, 0), "██████████", "se satura en 1.0");
    }

    #[test]
    fn area_uses_only_available_glyphs_and_right_aligns() {
        let mut b = Buffer::new(5, 2);
        area(
            &mut b,
            Rect::new(0, 0, 5, 2),
            &[1.0, 0.25, 0.0],
            theme::INK,
            theme::PAPER,
        );
        let col = |x| (b.get(x, 0).unwrap().ch, b.get(x, 1).unwrap().ch);
        // 3 muestras alineadas a la derecha: columnas 2, 3 y 4
        assert_eq!(col(2), ('█', '█'));
        assert_eq!(col(3), (' ', '▒'), "0.25 de 2 filas = media fila");
        assert_eq!(col(4), (' ', '░'), "línea base con 0");
        assert_eq!(col(0), (' ', ' '));
        let used: String = (0..5).flat_map(|x| [col(x).0, col(x).1]).collect();
        assert!(
            !used.contains(['▄', '▀', '▌', '▐', '▓']),
            "glifos ausentes en Terminus"
        );
    }

    #[test]
    fn leader_aligns_value_right() {
        let mut b = Buffer::new(16, 1);
        leader(&mut b, 0, 0, 16, "KERNEL", "7.2", theme::PAPER);
        assert_eq!(row(&b, 0), "KERNEL ····· 7.2");
    }

    #[test]
    fn text_clips() {
        let mut b = Buffer::new(8, 1);
        assert_eq!(
            text(&mut b, 2, 0, 3, "abcdef", style(theme::INK, theme::PAPER)),
            3
        );
        assert_eq!(row(&b, 0), "  abc   ");
    }
}
