//! Primitiva visual: una celda del terminal.

/// Color de una celda. `Reset` usa el color por defecto del terminal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Color {
    #[default]
    Reset,
    /// Paleta de 256 colores.
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// Atributos de texto como máscara de bits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Attrs(u8);

impl Attrs {
    pub const NONE: Attrs = Attrs(0);
    pub const BOLD: Attrs = Attrs(1);
    pub const DIM: Attrs = Attrs(1 << 1);
    pub const ITALIC: Attrs = Attrs(1 << 2);
    pub const UNDERLINE: Attrs = Attrs(1 << 3);
    pub const REVERSE: Attrs = Attrs(1 << 4);

    pub const fn union(self, other: Attrs) -> Attrs {
        Attrs(self.0 | other.0)
    }

    pub const fn contains(self, other: Attrs) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Aspecto de una celda (sin el carácter).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

impl Style {
    pub const fn new(fg: Color, bg: Color, attrs: Attrs) -> Style {
        Style { fg, bg, attrs }
    }
}

/// Una celda: carácter + estilo. La unidad de la UI lógica (filas × columnas).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Cell {
    /// Carácter de la segunda mitad de un carácter ancho; no se dibuja.
    pub const CONT: char = '\0';

    pub const fn new(ch: char, style: Style) -> Cell {
        Cell { ch, style }
    }

    /// Segunda celda de un carácter ancho (2 columnas).
    pub const fn continuation(style: Style) -> Cell {
        Cell::new(Cell::CONT, style)
    }

    pub const fn is_continuation(&self) -> bool {
        self.ch == Cell::CONT
    }

    /// Columnas que ocupa: 2 si es ancho, 1 en el resto (la continuación cuenta 1).
    pub fn width(&self) -> u16 {
        char_width(self.ch).max(1)
    }
}

/// Columnas de un carácter: 0 (combinante/control), 1 o 2 (CJK, emoji).
pub fn char_width(ch: char) -> u16 {
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0) as u16
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::new(' ', Style::default())
    }
}
