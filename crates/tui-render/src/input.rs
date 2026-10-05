//! Traduce eventos de crossterm a los de `tui-core`; lo que no encaja se descarta.

use crossterm::event::{
    Event as Ct, KeyCode as CtKey, KeyEventKind, KeyModifiers, MouseButton as CtBtn,
    MouseEventKind as CtMouse,
};
use tui_core::{Event, KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseKind};

fn mods(m: KeyModifiers) -> Modifiers {
    let mut out = Modifiers::NONE;
    for (from, to) in [
        (KeyModifiers::SHIFT, Modifiers::SHIFT),
        (KeyModifiers::CONTROL, Modifiers::CTRL),
        (KeyModifiers::ALT, Modifiers::ALT),
    ] {
        if m.contains(from) {
            out = out.union(to);
        }
    }
    out
}

fn button(b: CtBtn) -> MouseButton {
    match b {
        CtBtn::Left => MouseButton::Left,
        CtBtn::Right => MouseButton::Right,
        CtBtn::Middle => MouseButton::Middle,
    }
}

fn key(code: CtKey) -> Option<KeyCode> {
    Some(match code {
        CtKey::Char(c) => KeyCode::Char(c),
        CtKey::Enter => KeyCode::Enter,
        CtKey::Esc => KeyCode::Esc,
        CtKey::Tab => KeyCode::Tab,
        CtKey::BackTab => KeyCode::BackTab,
        CtKey::Backspace => KeyCode::Backspace,
        CtKey::Delete => KeyCode::Delete,
        CtKey::Insert => KeyCode::Insert,
        CtKey::Up => KeyCode::Up,
        CtKey::Down => KeyCode::Down,
        CtKey::Left => KeyCode::Left,
        CtKey::Right => KeyCode::Right,
        CtKey::Home => KeyCode::Home,
        CtKey::End => KeyCode::End,
        CtKey::PageUp => KeyCode::PageUp,
        CtKey::PageDown => KeyCode::PageDown,
        CtKey::F(n) => KeyCode::F(n),
        _ => return None,
    })
}

/// `None` para lo que TUI-OS no modela (foco, pegado, teclas multimedia, scroll lateral, sueltas de tecla).
pub fn convert(ev: Ct) -> Option<Event> {
    match ev {
        Ct::Key(k) if k.kind != KeyEventKind::Release => {
            Some(Event::Key(KeyEvent::new(key(k.code)?, mods(k.modifiers))))
        }
        Ct::Mouse(m) => {
            let kind = match m.kind {
                CtMouse::Down(b) => MouseKind::Down(button(b)),
                CtMouse::Up(b) => MouseKind::Up(button(b)),
                CtMouse::Drag(b) => MouseKind::Drag(button(b)),
                CtMouse::Moved => MouseKind::Moved,
                CtMouse::ScrollUp => MouseKind::ScrollUp,
                CtMouse::ScrollDown => MouseKind::ScrollDown,
                _ => return None,
            };
            Some(Event::Mouse(MouseEvent {
                kind,
                x: m.column,
                y: m.row,
                mods: mods(m.modifiers),
            }))
        }
        Ct::Resize(width, height) => Some(Event::Resize { width, height }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent as CtKeyEvent, MouseEvent as CtMouseEvent};

    #[test]
    fn ctrl_q() {
        let e = Ct::Key(CtKeyEvent::new(CtKey::Char('q'), KeyModifiers::CONTROL));
        assert_eq!(
            convert(e),
            Some(Event::Key(KeyEvent::new(
                KeyCode::Char('q'),
                Modifiers::CTRL
            )))
        );
    }

    #[test]
    fn function_and_navigation_keys() {
        let f1 = Ct::Key(CtKeyEvent::new(CtKey::F(1), KeyModifiers::NONE));
        assert_eq!(
            convert(f1),
            Some(Event::Key(KeyEvent::new(KeyCode::F(1), Modifiers::NONE)))
        );
        let bt = Ct::Key(CtKeyEvent::new(CtKey::BackTab, KeyModifiers::SHIFT));
        assert_eq!(
            convert(bt),
            Some(Event::Key(KeyEvent::new(
                KeyCode::BackTab,
                Modifiers::SHIFT
            )))
        );
    }

    #[test]
    fn key_release_is_dropped_and_unmodelled_keys_too() {
        let mut up = CtKeyEvent::new(CtKey::Char('a'), KeyModifiers::NONE);
        up.kind = KeyEventKind::Release;
        assert_eq!(convert(Ct::Key(up)), None);
        assert_eq!(
            convert(Ct::Key(CtKeyEvent::new(
                CtKey::CapsLock,
                KeyModifiers::NONE
            ))),
            None
        );
        assert_eq!(convert(Ct::FocusGained), None);
        assert_eq!(convert(Ct::Paste("x".into())), None);
    }

    #[test]
    fn mouse_uses_cell_coordinates() {
        let m = Ct::Mouse(CtMouseEvent {
            kind: CtMouse::Down(CtBtn::Left),
            column: 12,
            row: 3,
            modifiers: KeyModifiers::ALT,
        });
        assert_eq!(
            convert(m),
            Some(Event::Mouse(MouseEvent {
                kind: MouseKind::Down(MouseButton::Left),
                x: 12,
                y: 3,
                mods: Modifiers::ALT
            }))
        );
        let side = Ct::Mouse(CtMouseEvent {
            kind: CtMouse::ScrollLeft,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(convert(side), None);
    }

    #[test]
    fn resize_is_an_event() {
        assert_eq!(
            convert(Ct::Resize(120, 40)),
            Some(Event::Resize {
                width: 120,
                height: 40
            })
        );
    }
}
