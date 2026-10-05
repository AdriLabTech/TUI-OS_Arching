//! Núcleo del runtime TUI de TUI-OS: celdas, buffers y superficies.
//!
//! La UI lógica trabaja en filas × columnas; el tamaño lo da siempre el backend
//! de render (M2-02), nunca se fija aquí.

pub mod buffer;
pub mod cell;
pub mod event;
pub mod geometry;
pub mod surface;

pub use buffer::Buffer;
pub use cell::{Attrs, Cell, Color, Style};
pub use event::{Event, KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent, MouseKind};
pub use geometry::Rect;
pub use surface::Surface;

/// Nombre del sistema, para banners y `tui system info`.
pub const OS_NAME: &str = "TUI-OS";

#[cfg(test)]
mod tests;
