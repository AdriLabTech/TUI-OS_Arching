//! Rejilla de celdas con recorte seguro: nunca entra en pánico por coordenadas fuera.

use crate::cell::{Cell, Style};

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
                self.cells[i] = cell;
                true
            }
            None => false,
        }
    }

    pub fn fill(&mut self, cell: Cell) {
        self.cells.fill(cell);
    }

    /// Escribe `text` en la fila `y` desde `x`, recortando por la derecha.
    /// Devuelve las celdas escritas. Un carácter = una celda (ver deuda: ancho Unicode).
    pub fn put_str(&mut self, x: u16, y: u16, text: &str, style: Style) -> u16 {
        let mut written = 0;
        for (i, ch) in text.chars().enumerate() {
            let Some(cx) = u16::try_from(i).ok().and_then(|i| x.checked_add(i)) else {
                break;
            };
            if !self.set(cx, y, Cell::new(ch, style)) {
                break;
            }
            written += 1;
        }
        written
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
                    self.set(
                        x,
                        y,
                        src.cells[sy as usize * src.width as usize + sx as usize],
                    );
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
