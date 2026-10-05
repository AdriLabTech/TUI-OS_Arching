//! Superficie: un `Buffer` donde dibuja una aplicación, con seguimiento de daño.
//! La posición, el z-order y el foco pertenecen al compositor (M3), no a la superficie.

use crate::buffer::Buffer;
use crate::cell::{Cell, Style};
use crate::geometry::Rect;

#[derive(Clone, Debug)]
pub struct Surface {
    buffer: Buffer,
    damage: Rect,
}

impl Surface {
    /// Nace totalmente dañada: hay que pintarla entera la primera vez.
    pub fn new(width: u16, height: u16) -> Surface {
        Surface {
            buffer: Buffer::new(width, height),
            damage: Rect::new(0, 0, width, height),
        }
    }

    pub fn width(&self) -> u16 {
        self.buffer.width()
    }

    pub fn height(&self) -> u16 {
        self.buffer.height()
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if self.buffer.set(x, y, cell) {
            self.damage = self.damage.union(Rect::new(x, y, 1, 1));
        }
    }

    pub fn put_str(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let n = self.buffer.put_str(x, y, text, style);
        self.damage = self.damage.union(Rect::new(x, y, n, 1));
    }

    pub fn fill(&mut self, cell: Cell) {
        self.buffer.fill(cell);
        self.damage = Rect::new(0, 0, self.width(), self.height());
    }

    /// Un resize es un evento normal; deja la superficie entera marcada como dañada.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer.resize(width, height);
        self.damage = Rect::new(0, 0, width, height);
    }

    /// Rectángulo dañado desde la última composición (vacío si no hay).
    pub fn damage(&self) -> Rect {
        self.damage
    }

    /// Devuelve el rectángulo dañado desde la última llamada y lo reinicia.
    pub fn take_damage(&mut self) -> Rect {
        std::mem::take(&mut self.damage)
    }
}
