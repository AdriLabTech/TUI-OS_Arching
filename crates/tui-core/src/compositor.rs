//! Compositor: posiciona superficies en pantalla, gestiona z-order, foco, recorte y
//! enruta eventos. Las superficies no conocen su posición; esta vive aquí.

use crate::buffer::Buffer;
use crate::cell::Cell;
use crate::event::{Event, MouseEvent, MouseKind};
use crate::event::{KeyCode, Modifiers};
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
    /// Lo ha consumido el propio compositor (p. ej. ALT+TAB).
    Consumed,
    /// Ningún destino: ratón sobre el fondo, o sin ventanas para el teclado.
    None,
}

/// Cómo se colocan las ventanas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// Posición libre y z-order; un clic sube la ventana.
    #[default]
    Floating,
    /// Mosaico automático sin solapes: la primera ventana a la izquierda (maestra) y el
    /// resto apiladas a la derecha. El orden es el de creación; el foco no lo cambia.
    Tiled,
}

#[derive(Default)]
pub struct Compositor {
    /// De atrás hacia delante: la última es la superior.
    windows: Vec<Window>,
    next_id: u32,
    focus: Option<WindowId>,
    layout: Layout,
    background: Cell,
    /// Hay que volver a componer (cambio estructural o daño en alguna superficie).
    dirty: bool,
    last_size: (u16, u16),
}

impl Compositor {
    pub fn new() -> Compositor {
        Compositor::default()
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
        self.dirty = true;
    }

    pub fn set_background(&mut self, background: Cell) {
        self.background = background;
        self.dirty = true;
    }

    /// Añade una ventana encima de todas y le da el foco.
    pub fn add(&mut self, x: i32, y: i32, surface: Surface) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        self.windows.push(Window { id, x, y, surface });
        self.focus = Some(id);
        self.dirty = true;
        id
    }

    /// Quita la ventana; si tenía el foco pasa a la superior de las restantes.
    pub fn remove(&mut self, id: WindowId) -> Option<Surface> {
        let i = self.position(id)?;
        let w = self.windows.remove(i);
        if self.focus == Some(id) {
            self.focus = self.windows.last().map(|w| w.id);
        }
        self.dirty = true;
        Some(w.surface)
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
            self.dirty = true;
        }
    }

    /// Sube la ventana a lo alto del z-order y le da el foco.
    pub fn raise(&mut self, id: WindowId) {
        if let Some(i) = self.position(id) {
            let w = self.windows.remove(i);
            self.windows.push(w);
            self.focus = Some(id);
            self.dirty = true;
        }
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focus
    }

    /// Da el foco sin tocar el orden (en mosaico el orden es la disposición).
    pub fn set_focus(&mut self, id: WindowId) {
        if self.position(id).is_some() {
            self.focus = Some(id);
        }
    }

    /// Foco a la ventana siguiente (o anterior) en orden de creación, dando la vuelta.
    pub fn cycle_focus(&mut self, forward: bool) {
        let n = self.windows.len();
        if n == 0 {
            return;
        }
        let cur = self.focus.and_then(|id| self.position(id)).unwrap_or(0);
        let next = if forward {
            (cur + 1) % n
        } else {
            (cur + n - 1) % n
        };
        self.focus = Some(self.windows[next].id);
    }

    /// Ventana visible más alta bajo la celda `(x, y)`.
    pub fn window_at(&self, x: u16, y: u16) -> Option<WindowId> {
        self.windows
            .iter()
            .rev()
            .find(|w| w.contains(x, y))
            .map(|w| w.id)
    }

    /// ¿Habría que recomponer? (cambio estructural o superficies dañadas).
    pub fn needs_compose(&self) -> bool {
        self.dirty || self.windows.iter().any(|w| !w.surface.damage().is_empty())
    }

    /// Dibuja fondo y ventanas (de atrás hacia delante, recortadas a `out`) si hace falta.
    /// `out` debe ser siempre el mismo búfer entre llamadas: si nada ha cambiado no se toca
    /// y devuelve `false`. En mosaico recoloca (y redimensiona superficies) según `out`.
    pub fn compose(&mut self, out: &mut Buffer) -> bool {
        let size = (out.width(), out.height());
        if size != self.last_size {
            self.last_size = size;
            self.dirty = true;
        }
        if !self.needs_compose() {
            return false;
        }
        if self.layout == Layout::Tiled {
            self.tile(Rect::new(0, 0, size.0, size.1));
        }
        out.fill(self.background);
        for w in &mut self.windows {
            w.surface.take_damage();
            out.blit(w.surface.buffer(), w.x, w.y);
        }
        self.dirty = false;
        true
    }

    /// Master a la izquierda (60 % del ancho) y el resto apiladas a la derecha.
    fn tile(&mut self, screen: Rect) {
        let n = self.windows.len();
        if n == 0 || screen.is_empty() {
            return;
        }
        let (master_w, side_w) = if n == 1 {
            (screen.width, 0)
        } else {
            let m = (screen.width as u32 * 6 / 10).max(1) as u16;
            (
                m.min(screen.width - 1).max(1),
                screen.width - m.min(screen.width - 1).max(1),
            )
        };
        let rows = (n - 1).max(1) as u16;
        for (i, w) in self.windows.iter_mut().enumerate() {
            let (x, y, width, height) = if i == 0 {
                (screen.x, screen.y, master_w, screen.height)
            } else {
                let k = (i - 1) as u16;
                let (y0, y1) = (k * screen.height / rows, (k + 1) * screen.height / rows);
                (screen.x + master_w, screen.y + y0, side_w, y1 - y0)
            };
            w.x = x as i32;
            w.y = y as i32;
            if (w.surface.width(), w.surface.height()) != (width, height) {
                w.surface.resize(width, height);
            }
        }
    }

    /// Enruta un evento. ALT+TAB / ALT+SHIFT+TAB cambian el foco (lo consume el compositor).
    /// El resto del teclado va a la ventana con foco; el ratón, a la ventana bajo el cursor
    /// con coordenadas locales (un clic la enfoca y, en modo libre, la sube). `Resize` no se
    /// enruta: lo gestiona quien posee el compositor.
    pub fn route(&mut self, event: Event) -> Routed {
        match event {
            Event::Key(k) if k.mods.contains(Modifiers::ALT) && is_tab(k.code) => {
                self.cycle_focus(k.code == KeyCode::Tab && !k.mods.contains(Modifiers::SHIFT));
                Routed::Consumed
            }
            Event::Key(_) => match self.focus {
                Some(id) => Routed::Window(id, event),
                None => Routed::None,
            },
            Event::Mouse(m) => {
                let Some(id) = self.window_at(m.x, m.y) else {
                    return Routed::None;
                };
                if matches!(m.kind, MouseKind::Down(_)) {
                    match self.layout {
                        Layout::Floating => self.raise(id),
                        Layout::Tiled => self.set_focus(id),
                    }
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

fn is_tab(code: KeyCode) -> bool {
    matches!(code, KeyCode::Tab | KeyCode::BackTab)
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

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::cell::Style;
    use crate::event::{KeyEvent, MouseButton};

    fn win(c: &mut Compositor) -> WindowId {
        c.add(0, 0, Surface::new(1, 1))
    }

    fn alt_tab(shift: bool) -> Event {
        let mods = if shift {
            Modifiers::ALT.union(Modifiers::SHIFT)
        } else {
            Modifiers::ALT
        };
        Event::Key(KeyEvent::new(KeyCode::Tab, mods))
    }

    #[test]
    fn compose_paints_background_and_leaves_no_trails() {
        let mut c = Compositor::new();
        c.set_background(Cell::new('.', Style::default()));
        let a = c.add(0, 0, Surface::new(2, 2));
        let mut out = Buffer::new(6, 3);
        assert!(c.compose(&mut out));
        c.move_to(a, 4, 1);
        assert!(c.compose(&mut out));
        assert_eq!(out.get(0, 0).unwrap().ch, '.');
        c.remove(a);
        assert!(c.compose(&mut out));
        assert!((0..6).all(|x| (0..3).all(|y| out.get(x, y).unwrap().ch == '.')));
    }

    #[test]
    fn compose_skips_when_clean_and_reacts_to_damage_and_resize() {
        let mut c = Compositor::new();
        let a = win(&mut c);
        let mut out = Buffer::new(4, 4);
        assert!(c.compose(&mut out));
        assert!(!c.compose(&mut out));
        c.surface_mut(a)
            .unwrap()
            .set(0, 0, Cell::new('x', Style::default()));
        assert!(c.needs_compose() && c.compose(&mut out));
        assert_eq!(out.get(0, 0).unwrap().ch, 'x');
        let mut bigger = Buffer::new(5, 4);
        assert!(c.compose(&mut bigger));
    }

    #[test]
    fn alt_tab_cycles_focus_without_reordering() {
        let mut c = Compositor::new();
        let (a, b, d) = (win(&mut c), win(&mut c), win(&mut c));
        assert_eq!(c.focused(), Some(d));
        assert_eq!(c.route(alt_tab(false)), Routed::Consumed);
        assert_eq!(c.focused(), Some(a));
        c.route(alt_tab(false));
        assert_eq!(c.focused(), Some(b));
        c.route(alt_tab(true));
        assert_eq!(c.focused(), Some(a));
        c.route(alt_tab(true));
        assert_eq!(c.focused(), Some(d));
        assert_eq!(c.window_at(0, 0), Some(d)); // z-order intacto
    }

    #[test]
    fn tiled_layout_has_no_gaps_or_overlaps_and_follows_resize() {
        for n in 1..=5usize {
            let mut c = Compositor::new();
            c.set_layout(Layout::Tiled);
            let ids: Vec<_> = (0..n).map(|_| win(&mut c)).collect();
            for (w, h) in [(80u16, 24u16), (7, 5)] {
                let mut out = Buffer::new(w, h);
                c.compose(&mut out);
                let mut owner = vec![0u8; w as usize * h as usize];
                for id in &ids {
                    let r = c.visible_rect(*id, Rect::new(0, 0, w, h)).unwrap();
                    for y in r.y..r.y + r.height {
                        for x in r.x..r.x + r.width {
                            owner[y as usize * w as usize + x as usize] += 1;
                        }
                    }
                }
                assert!(owner.iter().all(|&o| o == 1), "n={n} {w}x{h}");
            }
        }
    }

    #[test]
    fn tiled_click_focuses_without_raising() {
        let mut c = Compositor::new();
        c.set_layout(Layout::Tiled);
        let (a, b) = (win(&mut c), win(&mut c));
        let mut out = Buffer::new(20, 10);
        c.compose(&mut out);
        let click = Event::Mouse(MouseEvent {
            kind: MouseKind::Down(MouseButton::Left),
            x: 1,
            y: 1,
            mods: Modifiers::NONE,
        });
        assert!(matches!(c.route(click), Routed::Window(id, _) if id == a));
        assert_eq!(c.focused(), Some(a));
        assert_eq!(c.window_at(15, 1), Some(b));
    }
}
