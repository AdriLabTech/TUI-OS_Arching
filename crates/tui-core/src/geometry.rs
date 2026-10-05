//! Geometría en celdas (nunca en píxeles).

/// Rectángulo en coordenadas de celda. Con `width == 0` o `height == 0` está vacío.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Rectángulo mínimo que contiene a ambos; el vacío no aporta nada.
    pub fn union(self, other: Rect) -> Rect {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x as u32 + self.width as u32).max(other.x as u32 + other.width as u32);
        let y1 = (self.y as u32 + self.height as u32).max(other.y as u32 + other.height as u32);
        let span = |hi: u32, lo: u16| u16::try_from(hi - lo as u32).unwrap_or(u16::MAX);
        Rect::new(x0, y0, span(x1, x0), span(y1, y0))
    }
}
