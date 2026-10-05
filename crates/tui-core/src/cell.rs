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
    pub const fn new(ch: char, style: Style) -> Cell {
        Cell { ch, style }
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::new(' ', Style::default())
    }
}
