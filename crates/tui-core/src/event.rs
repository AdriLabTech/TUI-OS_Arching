//! Eventos de entrada, independientes del backend de terminal.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Modifiers = Modifiers(0);
    pub const SHIFT: Modifiers = Modifiers(1);
    pub const CTRL: Modifiers = Modifiers(1 << 1);
    pub const ALT: Modifiers = Modifiers(1 << 2);

    pub const fn union(self, other: Modifiers) -> Modifiers {
        Modifiers(self.0 | other.0)
    }

    pub const fn contains(self, other: Modifiers) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    /// Teclas de función: `F(1)` = F1.
    F(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub mods: Modifiers,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, mods: Modifiers) -> KeyEvent {
        KeyEvent { code, mods }
    }

    /// ¿Es exactamente esta combinación? (p. ej. `is(KeyCode::Char('q'), Modifiers::CTRL)`)
    pub fn is(&self, code: KeyCode, mods: Modifiers) -> bool {
        self.code == code && self.mods == mods
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseKind {
    Down(MouseButton),
    Up(MouseButton),
    Drag(MouseButton),
    Moved,
    ScrollUp,
    ScrollDown,
}

/// Posición en celdas, no en píxeles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseKind,
    pub x: u16,
    pub y: u16,
    pub mods: Modifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    /// Nuevo tamaño lógico: columnas × filas.
    Resize {
        width: u16,
        height: u16,
    },
}
