# ADR-0003: Terminal embebido (PTY + intérprete VT)

**Estado:** aceptada (2026-10-07, spike de M5-01)
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

## Resultado del spike (M5-01, 2026-10-07)
- `portable-pty` 0.9 compila y funciona en Linux sin `unsafe` propio; el hilo lector y el cierre (EOF/EIO al salir el hijo) se comportan bien.
- `vt100` 0.16.2 sirve para shell, `top`, `btop` y `yazi`, pero **no implementa HVP (`CSI f`)**: btop salía corrupto. Se parcheó con un normalizador de 60 líneas (`compat`). Tampoco implementa REP, `CSI s/u` ni respuestas a consultas.
- Decisión: se queda `vt100` + `compat`. **Se pasa a `vte` + modelo propio** si hace falta parchear otra secuencia (el modelo de pantalla sería ~500 líneas nuestras) o cuando M5-02 necesite grafemas completos por celda, que `vt100` entrega como `String` pero nuestra `Cell` no admite.
- Sin verificar: `htop`/`vim` (no instalados), el target QEMU y `cargo audit`.
