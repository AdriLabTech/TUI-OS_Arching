//! Renderer de TUI-OS: pinta un `Buffer` en un terminal emitiendo solo lo que cambió.
//!
//! `Renderer` genera secuencias ANSI sobre cualquier `Write` (testeable con un `Vec<u8>`);
//! `Terminal` gestiona el modo raw/pantalla alternativa y el tamaño reales.

pub mod input;
mod renderer;
mod terminal;

pub use renderer::{Palette, Renderer};
pub use terminal::Terminal;
