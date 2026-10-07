//! Codificación de teclas a los bytes que espera un programa en un terminal.

use tui_core::{KeyCode, KeyEvent, Modifiers};

/// Bytes que se escriben en el PTY para `key`. `app_cursor` es el modo DECCKM del
/// programa (flechas como `ESC O A` en vez de `ESC [ A`). Vacío si la tecla no tiene
/// representación.
pub fn encode(key: KeyEvent, app_cursor: bool) -> Vec<u8> {
    let KeyEvent { code, mods } = key;
    let alt = mods.contains(Modifiers::ALT);
    let ctrl = mods.contains(Modifiers::CTRL);
    let shift = mods.contains(Modifiers::SHIFT);

    // Parámetro xterm de modificadores: 1 + shift(1) + alt(2) + ctrl(4).
    let modifier = 1 + shift as u8 + 2 * alt as u8 + 4 * ctrl as u8;
    let csi = |final_byte: char| -> Vec<u8> {
        if modifier == 1 {
            format!("\x1b[{final_byte}").into_bytes()
        } else {
            format!("\x1b[1;{modifier}{final_byte}").into_bytes()
        }
    };
    let arrow = |final_byte: char| -> Vec<u8> {
        if modifier == 1 && app_cursor {
            format!("\x1bO{final_byte}").into_bytes()
        } else {
            csi(final_byte)
        }
    };
    let tilde = |n: u8| -> Vec<u8> {
        if modifier == 1 {
            format!("\x1b[{n}~").into_bytes()
        } else {
            format!("\x1b[{n};{modifier}~").into_bytes()
        }
    };

    let mut out = match code {
        KeyCode::Char(c) if ctrl => match c {
            'a'..='z' | 'A'..='Z' => vec![(c.to_ascii_lowercase() as u8) & 0x1f],
            '@' | ' ' => vec![0],
            '[' => vec![0x1b],
            '\\' => vec![0x1c],
            ']' => vec![0x1d],
            '^' => vec![0x1e],
            '_' => vec![0x1f],
            _ => return Vec::new(),
        },
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Delete => tilde(3),
        KeyCode::Insert => tilde(2),
        KeyCode::Up => arrow('A'),
        KeyCode::Down => arrow('B'),
        KeyCode::Right => arrow('C'),
        KeyCode::Left => arrow('D'),
        KeyCode::Home => csi('H'),
        KeyCode::End => csi('F'),
        KeyCode::PageUp => tilde(5),
        KeyCode::PageDown => tilde(6),
        KeyCode::F(n @ 1..=4) => {
            let f = b"PQRS"[n as usize - 1] as char;
            if modifier == 1 {
                format!("\x1bO{f}").into_bytes()
            } else {
                csi(f)
            }
        }
        KeyCode::F(n @ 5..=12) => {
            let code = [15, 17, 18, 19, 20, 21, 23, 24][n as usize - 5];
            tilde(code)
        }
        KeyCode::F(_) => return Vec::new(),
    };

    // Alt sobre un carácter o una tecla de control se manda como ESC + bytes. Las teclas
    // con secuencia CSI ya llevan Alt en el parámetro de modificadores.
    if alt
        && matches!(
            code,
            KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab | KeyCode::Backspace
        )
    {
        out.insert(0, 0x1b);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(code: KeyCode, mods: Modifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn texto_y_control() {
        assert_eq!(encode(k(KeyCode::Char('a'), Modifiers::NONE), false), b"a");
        assert_eq!(
            encode(k(KeyCode::Char('ñ'), Modifiers::NONE), false),
            "ñ".as_bytes()
        );
        assert_eq!(encode(k(KeyCode::Char('c'), Modifiers::CTRL), false), [3]);
        assert_eq!(
            encode(k(KeyCode::Char('['), Modifiers::CTRL), false),
            [0x1b]
        );
        assert_eq!(encode(k(KeyCode::Enter, Modifiers::NONE), false), b"\r");
        assert_eq!(
            encode(k(KeyCode::Backspace, Modifiers::NONE), false),
            [0x7f]
        );
    }

    #[test]
    fn alt_antepone_escape() {
        assert_eq!(
            encode(k(KeyCode::Char('b'), Modifiers::ALT), false),
            b"\x1bb"
        );
    }

    #[test]
    fn flechas_segun_modo() {
        assert_eq!(encode(k(KeyCode::Up, Modifiers::NONE), false), b"\x1b[A");
        assert_eq!(encode(k(KeyCode::Up, Modifiers::NONE), true), b"\x1bOA");
        // Con modificadores siempre va en CSI.
        assert_eq!(
            encode(k(KeyCode::Left, Modifiers::CTRL), true),
            b"\x1b[1;5D"
        );
    }

    #[test]
    fn teclas_de_funcion_y_edicion() {
        assert_eq!(encode(k(KeyCode::F(1), Modifiers::NONE), false), b"\x1bOP");
        assert_eq!(
            encode(k(KeyCode::F(5), Modifiers::NONE), false),
            b"\x1b[15~"
        );
        assert_eq!(
            encode(k(KeyCode::F(12), Modifiers::NONE), false),
            b"\x1b[24~"
        );
        assert_eq!(
            encode(k(KeyCode::Delete, Modifiers::NONE), false),
            b"\x1b[3~"
        );
        assert_eq!(
            encode(k(KeyCode::PageDown, Modifiers::SHIFT), false),
            b"\x1b[6;2~"
        );
        assert_eq!(
            encode(k(KeyCode::BackTab, Modifiers::SHIFT), false),
            b"\x1b[Z"
        );
    }
}
