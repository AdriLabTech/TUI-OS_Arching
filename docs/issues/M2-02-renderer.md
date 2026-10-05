# [M2-02] Renderer a terminal con tamaño y resize

**Milestone:** M2 · **Estado:** CLOSED (verificación en el target pendiente) · **Depende de:** M2-01

## Objetivo
Pintar un `Buffer` en un terminal emitiendo solo diferencias; el tamaño lo da el terminal.

## Alcance
Crate `tui-render` con `crossterm 0.29` (única dependencia nueva; API comprobada en su código fuente):
- `Renderer<W: Write>`: diff contra el fotograma anterior, `MoveTo` solo si hay salto, estilo solo si cambia, repintado completo en el primer fotograma, al cambiar de tamaño o tras `invalidate()`.
- `Terminal`: raw mode + pantalla alternativa + cursor oculto; se restaura en `Drop` (también en pánico). `size()` devuelve columnas × filas reales.
- Ejemplo `examples/hello.rs`.
No entra: eventos de teclado/ratón/resize por señal (M2-03; ahora el resize se detecta consultando `size()`).

## Criterios de aceptación
- [x] Sin dimensiones fijas; el resize repinta todo
- [x] Un fotograma idéntico no emite celdas; un cambio de una celda emite un `MoveTo` y un `Print`
- [x] El terminal se restaura al salir

## Verificación
- VERIFIED (1-2): fmt, clippy `-D warnings`, 6 tests nuevos del renderer (18 en total)
- VERIFIED (3, en el HOST): `hello` ejecutado en un pty: detecta `80x24`, y tras cambiar el pty a `120x40` redibuja con `size=120x40`; 2 borrados completos; entra y sale de la pantalla alternativa y restaura el cursor
- UNVERIFIED (4-5): ejecución dentro de TUI-OS (no hay aún un binario ni una sesión TUI en la imagen; llega con M4)
- Nota: un test propio tenía una aserción casi vacía; se sustituyó por una exacta (`"\x1b[0m"`).

## Limitaciones / deuda
- Sin ancho Unicode (heredado de M2-01); en la última columna se evita asumir la posición del cursor.
- `Renderer::render` clona el `Buffer` entero cada fotograma; medir antes de optimizar.
- Detección de resize por sondeo, hasta M2-03.

## Dependen de este
M2-03, M3.
