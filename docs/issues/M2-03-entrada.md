# [M2-03] Entrada: teclado, ratón y resize como eventos

**Milestone:** M2 · **Estado:** CLOSED (verificación en el target pendiente) · **Depende de:** M2-02

## Objetivo
Un modelo de eventos propio, independiente del backend, y resize como evento normal.

## Alcance
- `tui-core::event`: `Event` (`Key`, `Mouse`, `Resize{width,height}`), `KeyEvent`/`KeyCode`/`Modifiers`, `MouseEvent` (coordenadas en celdas).
- `tui-render::input::convert`: crossterm → eventos de `tui-core`; descarta lo no modelado (foco, pegado, teclas multimedia, scroll lateral, soltar tecla).
- `Terminal::poll_event(timeout)` y captura de ratón (se activa y se restaura en `Drop`). Sustituye el sondeo de tamaño.
No entra: enrutado de eventos y foco (compositor, M3); atajos globales (F1, ALT+TAB: M5).

## Criterios de aceptación
- [x] Teclas (incl. F1, BackTab, Ctrl/Alt/Shift), ratón y resize llegan como `Event`
- [x] Apps y `tui-core` no dependen de crossterm
- [x] Al salir se restaura ratón, pantalla alternativa y cursor

## Verificación
- VERIFIED (1-2): fmt, clippy `-D warnings`, 25 tests (14 en `tui-core`, 11 en `tui-render`; 7 nuevos de conversión y eventos)
- VERIFIED (3, HOST, pty): 9 ejecuciones (3 × {`q`, Esc, Ctrl+Q}) de `hello`: tecla `a`, F1, clic SGR en (12,3), resize 80×24→120×40 por evento, salida con código 0, ratón/pantalla alternativa/cursor restaurados, sin procesos huérfanos
- NO EXPLICADO: una primera ejecución con mi arnés Python se colgó con Ctrl+Q; no se reprodujo ni aislada (3 teclas) ni en 9 ejecuciones del arnés corregido (con límite duro). Probable fallo del arnés, no confirmado.
- UNVERIFIED (4-5): dentro de TUI-OS (sin sesión TUI hasta M4)

## Limitaciones / deuda
- Sin `KeyEventKind` (repetición vs. pulsación) ni protocolo de teclado kitty: un `Esc` suelto depende del retardo de desambiguación de crossterm.
- Sin ratón de movimiento sin botón garantizado según terminal; sin pegado entre corchetes.

## Dependen de este
M3 (enrutado de eventos), M5.
