//! Tokens de color «papel y tinta»: negro, hueso cálido, una rampa de tonos y
//! acentos de marca contados (bermellón, oro). Sin degradados ni sombras.
//!
//! Los colores son `Rgb`. En la consola de Linux (16 colores) el renderer reprograma
//! la paleta con `PALETTE`, así que los tonos salen exactos. Los índices 0-7 se usan
//! como fondo; los 8-15 solo como texto.

use tui_core::{Attrs, Color, Style};
use tui_render::Palette;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

// Superficies (fondo): cuatro niveles de tono.
pub const PAPER: Color = rgb(0x00, 0x00, 0x00);
pub const RAISED: Color = rgb(0x0e, 0x0e, 0x0e);
pub const CONTAINER: Color = rgb(0x1b, 0x1b, 0x1b);
pub const HIGH: Color = rgb(0x2a, 0x2a, 0x2a);
// Marca.
pub const VERMILION: Color = rgb(0xe2, 0x34, 0x2a);
pub const GOLD: Color = rgb(0xd9, 0xa4, 0x41);
pub const INK: Color = rgb(0xcd, 0xc4, 0xba);
pub const LINE: Color = rgb(0x3a, 0x37, 0x33);
// Solo texto.
pub const INK_DIM: Color = rgb(0xb0, 0xa9, 0xa0);
pub const MUTED: Color = rgb(0x95, 0x8f, 0x87);
pub const FAINT: Color = rgb(0x7a, 0x75, 0x6e);
pub const PATINA: Color = rgb(0x70, 0xc2, 0xa3);
pub const ACTIVE: Color = rgb(0xe0, 0x56, 0x3b);
pub const BRIGHT: Color = rgb(0xf2, 0xec, 0xe4);
pub const ERROR: Color = rgb(0xe6, 0x45, 0x53);
pub const WHITE: Color = rgb(0xff, 0xff, 0xff);

/// Orden = índice de la paleta ANSI. Los 8 primeros son los únicos válidos como fondo.
const SLOTS: [Color; 16] = [
    PAPER, VERMILION, CONTAINER, GOLD, HIGH, RAISED, LINE, INK, // 0-7
    FAINT, ACTIVE, PATINA, MUTED, ERROR, BRIGHT, INK_DIM, WHITE, // 8-15
];

pub fn palette() -> Palette {
    let mut p = [(0, 0, 0); 16];
    for (i, c) in SLOTS.iter().enumerate() {
        if let Color::Rgb(r, g, b) = c {
            p[i] = (*r, *g, *b);
        }
    }
    Palette(p)
}

pub fn style(fg: Color, bg: Color) -> Style {
    Style::new(fg, bg, Attrs::NONE)
}
