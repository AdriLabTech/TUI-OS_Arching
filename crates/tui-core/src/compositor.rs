//! Compositor: posiciona superficies en pantalla, gestiona z-order, foco, recorte y
//! enruta eventos. Las superficies no conocen su posición; esta vive aquí.

use crate::buffer::Buffer;
use crate::event::{Event, MouseEvent, MouseKind};
use crate::geometry::Rect;
use crate::surface::Surface;

/// Identificador estable de una ventana (no se reutiliza).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowId(u32);

struct Window {
    id: WindowId,
    /// Esquina superior izquierda en pantalla; puede quedar parcial o totalmente fuera.
    x: i32,
    y: i32,
    surface: Surface,
}

impl Window {
    fn contains(&self, x: u16, y: u16) -> bool {
        let (px, py) = (x as i32 - self.x, y as i32 - self.y);
        px >= 0 && py >= 0 && px < self.surface.width() as i32 && py < self.surface.height() as i32
    }
}

/// Destino de un evento tras el enrutado. Las coordenadas del ratón van relativas a la ventana.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Routed {
    Window(WindowId, Event),
    /// Ningún destino: ratón sobre el fondo, o sin ventanas para el teclado.
    None,
}

#[derive(Default)]
pub struct Compositor {
    /// De atrás hacia delante: la última es la superior.
    windows: Vec<Window>,
    next_id: u32,
}

impl Compositor {
    pub fn new() -> Compositor {
        Compositor::default()
    }

    /// Añade una ventana encima de todas y le da el foco.
    pub fn add(&mut self, x: i32, y: i32, surface: Surface) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        self.windows.push(Window { id, x, y, surface });
        id
    }

    pub fn remove(&mut self, id: WindowId) -> Option<Surface> {
        let i = self.position(id)?;
        Some(self.windows.remove(i).surface)
    }

    fn position(&self, id: WindowId) -> Option<usize> {
        self.windows.iter().position(|w| w.id == id)
    }

    pub fn surface_mut(&mut self, id: WindowId) -> Option<&mut Surface> {
        let i = self.position(id)?;
        Some(&mut self.windows[i].surface)
    }

    pub fn move_to(&mut self, id: WindowId, x: i32, y: i32) {
        if let Some(i) = self.position(id) {
            self.windows[i].x = x;
            self.windows[i].y = y;
        }
    }

    /// Sube la ventana a lo alto del z-order (y por tanto le da el foco).
    pub fn raise(&mut self, id: WindowId) {
        if let Some(i) = self.position(id) {
            let w = self.windows.remove(i);
            self.windows.push(w);
        }
    }

    /// La ventana con foco es siempre la superior.
    pub fn focused(&self) -> Option<WindowId> {
        self.windows.last().map(|w| w.id)
    }

    /// Ventana visible más alta bajo la celda `(x, y)`.
    pub fn window_at(&self, x: u16, y: u16) -> Option<WindowId> {
        self.windows
            .iter()
            .rev()
            .find(|w| w.contains(x, y))
            .map(|w| w.id)
    }

    /// Dibuja todas las ventanas, de atrás hacia delante, recortadas a `out`.
    /// Consume el daño de las superficies (la composición es completa por ahora).
    pub fn compose(&mut self, out: &mut Buffer) {
        for w in &mut self.windows {
            w.surface.take_damage();
            out.blit(w.surface.buffer(), w.x, w.y);
        }
    }

    /// Enruta un evento: el teclado, a la ventana con foco; el ratón, a la ventana
    /// bajo el cursor (un clic además la sube y le da el foco). `Resize` no se enruta:
    /// lo gestiona quien posee el compositor.
    pub fn route(&mut self, event: Event) -> Routed {
        match event {
            Event::Key(_) => match self.focused() {
                Some(id) => Routed::Window(id, event),
                None => Routed::None,
            },
            Event::Mouse(m) => {
                let Some(id) = self.window_at(m.x, m.y) else {
                    return Routed::None;
                };
                if matches!(m.kind, MouseKind::Down(_)) {
                    self.raise(id);
                }
                let w = &self.windows[self.position(id).expect("ventana recién encontrada")];
                let local = MouseEvent {
                    x: (m.x as i32 - w.x) as u16,
                    y: (m.y as i32 - w.y) as u16,
                    ..m
                };
                Routed::Window(id, Event::Mouse(local))
            }
            Event::Resize { .. } => Routed::None,
        }
    }

    /// Rectángulo en pantalla de la ventana, recortado a `screen`; `None` si queda fuera.
    pub fn visible_rect(&self, id: WindowId, screen: Rect) -> Option<Rect> {
        let w = &self.windows[self.position(id)?];
        let x0 = w.x.max(screen.x as i32);
        let y0 = w.y.max(screen.y as i32);
        let x1 = (w.x + w.surface.width() as i32).min(screen.x as i32 + screen.width as i32);
        let y1 = (w.y + w.surface.height() as i32).min(screen.y as i32 + screen.height as i32);
        (x1 > x0 && y1 > y0)
            .then(|| Rect::new(x0 as u16, y0 as u16, (x1 - x0) as u16, (y1 - y0) as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{Cell, Style};
    use crate::event::{KeyCode, KeyEvent, Modifiers, MouseButton};

    fn solid(w: u16, h: u16, ch: char) -> Surface {
        let mut s = Surface::new(w, h);
        s.fill(Cell::new(ch, Style::default()));
        s
    }

    fn click(x: u16, y: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind: MouseKind::Down(MouseButton::Left),
            x,
            y,
            mods: Modifiers::NONE,
        })
    }

    fn key() -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char('a'), Modifiers::NONE))
    }

    fn ch(b: &Buffer, x: u16, y: u16) -> char {
        b.get(x, y).unwrap().ch
    }

    #[test]
    fn top_window_wins_and_is_focused() {
        let mut c = Compositor::new();
        let a = c.add(0, 0, solid(4, 4, 'a'));
        let b = c.add(2, 2, solid(4, 4, 'b'));
        assert_eq!(c.focused(), Some(b));
        let mut out = Buffer::new(8, 8);
        c.compose(&mut out);
        assert_eq!(ch(&out, 1, 1), 'a');
        assert_eq!(ch(&out, 2, 2), 'b');
        c.raise(a);
        c.compose(&mut out);
        assert_eq!(ch(&out, 2, 2), 'a');
        assert_eq!(c.focused(), Some(a));
    }

    #[test]
    fn clips_partially_and_totally_offscreen() {
        let mut c = Compositor::new();
        let a = c.add(-2, -1, solid(4, 3, 'a'));
        let far = c.add(100, 100, solid(4, 4, 'z'));
        c.add(6, 6, solid(4, 4, 'b'));
        let mut out = Buffer::new(8, 8);
        c.compose(&mut out); // no debe entrar en pánico
        assert_eq!(ch(&out, 1, 1), 'a');
        assert_eq!(ch(&out, 2, 0), ' ');
        assert_eq!(ch(&out, 7, 7), 'b');
        let screen = Rect::new(0, 0, 8, 8);
        assert_eq!(c.visible_rect(a, screen), Some(Rect::new(0, 0, 2, 2)));
        assert_eq!(c.visible_rect(far, screen), None);
    }

    #[test]
    fn keys_go_to_focused_window() {
        let mut c = Compositor::new();
        assert_eq!(c.route(key()), Routed::None);
        let a = c.add(0, 0, solid(2, 2, 'a'));
        c.add(5, 5, solid(2, 2, 'b'));
        c.raise(a);
        assert_eq!(c.route(key()), Routed::Window(a, key()));
    }

    #[test]
    fn click_hits_topmost_raises_and_localizes() {
        let mut c = Compositor::new();
        let a = c.add(0, 0, solid(6, 6, 'a'));
        let b = c.add(3, 3, solid(6, 6, 'b'));
        // Zona solapada: gana b. Local = (4,4) - (3,3) = (1,1).
        let Routed::Window(id, Event::Mouse(m)) = c.route(click(4, 4)) else {
            panic!("debía enrutarse");
        };
        assert_eq!((id, m.x, m.y), (b, 1, 1));
        // Clic solo sobre a: la sube y le da el foco.
        let Routed::Window(id, Event::Mouse(m)) = c.route(click(1, 2)) else {
            panic!("debía enrutarse");
        };
        assert_eq!((id, m.x, m.y), (a, 1, 2));
        assert_eq!(c.focused(), Some(a));
        // Fondo: sin destino.
        assert_eq!(c.route(click(20, 20)), Routed::None);
    }

    #[test]
    fn remove_passes_focus_down_and_ids_are_not_reused() {
        let mut c = Compositor::new();
        let a = c.add(0, 0, solid(2, 2, 'a'));
        let b = c.add(0, 0, solid(2, 2, 'b'));
        assert!(c.remove(b).is_some());
        assert_eq!(c.focused(), Some(a));
        assert!(c.remove(b).is_none());
        let n = c.add(0, 0, solid(1, 1, 'n'));
        assert_ne!(n, b);
    }
}
