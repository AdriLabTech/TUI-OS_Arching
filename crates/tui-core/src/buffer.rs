//! Rejilla de celdas con recorte seguro: nunca entra en pánico por coordenadas fuera.

use crate::cell::{Cell, Style, char_width};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Buffer {
        Buffer {
            width,
            height,
            cells: vec![Cell::default(); width as usize * height as usize],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        (x < self.width && y < self.height).then(|| y as usize * self.width as usize + x as usize)
    }

    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.cells[i])
    }

    /// Escribe una celda; fuera de la rejilla no hace nada. Devuelve si escribió.
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) -> bool {
        match self.index(x, y) {
            Some(i) => {
                // Pisar una mitad de un carácter ancho deja la otra mitad huérfana: se limpia.
                if self.cells[i].is_continuation() && x > 0 {
                    self.cells[i - 1] = Cell::new(' ', self.cells[i - 1].style);
                } else if char_width(self.cells[i].ch) == 2 && x + 1 < self.width {
                    self.cells[i + 1] = Cell::new(' ', self.cells[i + 1].style);
                }
                self.cells[i] = cell;
                true
            }
            None => false,
        }
    }

    pub fn fill(&mut self, cell: Cell) {
        self.cells.fill(cell);
    }

    /// Escribe `text` en la fila `y` desde `x`, recortando por la derecha. Devuelve las
    /// columnas escritas. Los caracteres anchos ocupan 2 celdas (la segunda es una
    /// continuación); si no caben entero se sustituyen por un espacio. Los de ancho 0
    /// (combinantes) se descartan: no hay agrupación en clústeres de grafemas (deuda).
    pub fn put_str(&mut self, x: u16, y: u16, text: &str, style: Style) -> u16 {
        let mut col = x as u32;
        for ch in text.chars() {
            let w = char_width(ch);
            if w == 0 {
                continue;
            }
            let Ok(cx) = u16::try_from(col) else { break };
            if w == 2 && cx.saturating_add(1) >= self.width {
                if self.set(cx, y, Cell::new(' ', style)) {
                    col += 1;
                }
                break;
            }
            if !self.set(cx, y, Cell::new(ch, style)) {
                break;
            }
            if w == 2 {
                self.set(cx + 1, y, Cell::continuation(style));
            }
            col += w as u32;
        }
        (col - x as u32) as u16
    }

    /// Cambia el tamaño conservando el contenido que siga cabiendo.
    pub fn resize(&mut self, width: u16, height: u16) {
        let mut next = Buffer::new(width, height);
        for y in 0..self.height.min(height) {
            for x in 0..self.width.min(width) {
                next.cells[y as usize * width as usize + x as usize] =
                    self.cells[y as usize * self.width as usize + x as usize];
            }
        }
        *self = next;
    }

    /// Copia `src` con su esquina en `(dx, dy)`, que puede ser negativa; recorta a la rejilla.
    pub fn blit(&mut self, src: &Buffer, dx: i32, dy: i32) {
        for sy in 0..src.height {
            for sx in 0..src.width {
                let (x, y) = (sx as i32 + dx, sy as i32 + dy);
                if let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) {
                    let mut cell = src.cells[sy as usize * src.width as usize + sx as usize];
                    // Un carácter ancho recortado por cualquiera de sus lados se queda en espacio.
                    let cut_cont = cell.is_continuation() && (sx == 0 || x == 0);
                    let cut_wide = char_width(cell.ch) == 2
                        && (sx + 1 >= src.width || x.saturating_add(1) >= self.width);
                    if cut_cont || cut_wide {
                        cell = Cell::new(' ', cell.style);
                    }
                    self.set(x, y, cell);
                }
            }
        }
    }

    /// Celdas de `self` distintas de `prev`, para que el renderer emita solo cambios.
    /// Si los tamaños difieren, devuelve todas las celdas de `self`.
    pub fn diff(&self, prev: &Buffer) -> Vec<(u16, u16, Cell)> {
        let same_size = self.width == prev.width && self.height == prev.height;
        let mut out = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y as usize * self.width as usize + x as usize;
                if !same_size || self.cells[i] != prev.cells[i] {
                    out.push((x, y, self.cells[i]));
                }
            }
        }
        out
    }
}
