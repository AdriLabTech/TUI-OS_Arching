//! Núcleo del runtime TUI de TUI-OS.
//!
//! Marcador del workspace (M0-02). Las primitivas (`Cell`, `Buffer`, `Surface`)
//! llegan con el issue M2-01.

/// Nombre del sistema, para banners y `tui system info`.
pub const OS_NAME: &str = "TUI-OS";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_name_is_set() {
        assert_eq!(OS_NAME, "TUI-OS");
    }
}
