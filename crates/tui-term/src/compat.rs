//! Normaliza secuencias que `vt100` 0.16 no implementa pero los programas usan.
//!
//! Hoy: HVP (`CSI Pl ; Pc f`), idéntica a CUP (`CSI Pl ; Pc H`). btop la usa para
//! todo su dibujo y sin esto la pantalla sale desplazada y corrupta (spike de M5-01).

/// Longitud máxima de una secuencia CSI que se retiene esperando su final.
const MAX_HELD: usize = 64;

#[derive(Default)]
pub(crate) struct Compat {
    /// Inicio de una CSI cortada al final del fragmento anterior.
    held: Vec<u8>,
}

impl Compat {
    pub(crate) fn filter(&mut self, input: &[u8]) -> Vec<u8> {
        let mut data = std::mem::take(&mut self.held);
        data.extend_from_slice(input);
        let mut out = Vec::with_capacity(data.len());
        let mut i = 0;
        while i < data.len() {
            if data[i] != 0x1b {
                out.push(data[i]);
                i += 1;
                continue;
            }
            match data.get(i + 1) {
                None => {
                    self.held = data[i..].to_vec();
                    break;
                }
                Some(b'[') => {}
                Some(_) => {
                    out.push(0x1b);
                    i += 1;
                    continue;
                }
            }
            // Parámetros (0x30-0x3F) e intermedios (0x20-0x2F) hasta el byte final.
            let mut j = i + 2;
            while j < data.len() && matches!(data[j], 0x20..=0x3f) {
                j += 1;
            }
            if j == data.len() {
                if j - i > MAX_HELD {
                    out.extend_from_slice(&data[i..]);
                } else {
                    self.held = data[i..].to_vec();
                }
                break;
            }
            let params = &data[i + 2..j];
            let plain = params.iter().all(|b| matches!(b, b'0'..=b'9' | b';'));
            out.extend_from_slice(&data[i..j]);
            if data[j] == b'f' && plain {
                out.push(b'H');
            } else {
                out.push(data[j]);
            }
            i = j + 1;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hvp_pasa_a_cup() {
        let mut c = Compat::default();
        assert_eq!(c.filter(b"a\x1b[3;7fb\x1b[fc"), b"a\x1b[3;7Hb\x1b[Hc");
    }

    #[test]
    fn no_toca_otras_secuencias_ni_texto() {
        let mut c = Compat::default();
        let s = b"\x1b[1;31mhola\x1b[2J\x1b]0;f\x07 f \x1b[?25l\x1b7";
        assert_eq!(c.filter(s), s);
    }

    #[test]
    fn secuencia_cortada_entre_fragmentos() {
        let mut c = Compat::default();
        let mut out = c.filter(b"x\x1b");
        out.extend(c.filter(b"[12;"));
        out.extend(c.filter(b"4"));
        out.extend(c.filter(b"fy"));
        assert_eq!(out, b"x\x1b[12;4Hy");
    }

    #[test]
    fn csi_con_marcador_privado_o_intermedio_no_es_hvp() {
        let mut c = Compat::default();
        assert_eq!(c.filter(b"\x1b[?1f\x1b[1 f"), b"\x1b[?1f\x1b[1 f");
    }
}
