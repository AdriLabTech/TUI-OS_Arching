# ADR-0003: Terminal embebido (PTY + intérprete VT)

**Estado:** propuesta (se acepta al abrir M5-01)
**Contexto:** M5 añade `tui-term`, una ventana del compositor que ejecuta programas reales (shell, `htop`, `vim`, `pacman`). Hacen falta dos piezas: un **PTY** (lanzar el hijo con terminal de control y tamaño) y un **intérprete VT** (secuencias de escape → rejilla de celdas).

**Opciones (metadatos de crates.io verificados el 2026-10-05; comportamiento interno sin verificar hasta el spike de M5-01):**

| Pieza | Opción | Licencia | Nota |
|---|---|---|---|
| PTY | `portable-pty` 0.9 (wezterm) | MIT | API de alto nivel (abrir par, `spawn`, resize). Multiplataforma, que aquí no necesitamos (target solo Linux). |
| PTY | `rustix` (ya en el árbol de caché) con `pty` + `Command::pre_exec` | Apache-2.0/MIT | Menos dependencias, pero hay que escribir `setsid`, `TIOCSCTTY` y resize a mano: más `unsafe` propio. |
| VT | `vt100` 0.16 | MIT | Parser + modelo de pantalla ya hechos; menos código nuestro. |
| VT | `vte` 0.15 | Apache-2.0/MIT | Solo el parser: el modelo de pantalla (cursor, scroll, alt screen, atributos) lo escribimos nosotros. Más control. |
| VT | `alacritty_terminal` 0.26 | Apache-2.0 | Completo, pero pesado y acoplado a Alacritty. |

**Decisión propuesta:** `portable-pty` para el PTY (evita `unsafe` propio) y empezar con `vt100` para el modelo de pantalla, mapeando su rejilla a `Cell`. Si el spike muestra que `vt100` no cubre algo necesario (alt screen en `vim`, ratón, ancho de grafemas), pasar a `vte` + modelo propio. Descartado `alacritty_terminal` por tamaño.

**Criterio del spike (M5-01):** ejecutar `bash`, `htop` y `vim` dentro de la ventana, con redimensionado, y comparar con el mismo programa en un terminal real. Si `portable-pty` arrastra dependencias que choquen con el target (glibc del rootfs, ver M2-05), reconsiderar `rustix`.

**Consecuencias:** dos dependencias nuevas con su superficie de CVE: revisar con `cargo audit` al añadirlas. El hijo corre con el usuario de la sesión (M4 antes de M5, para no abrir una shell como root).
