# [M5-01] Spike del terminal embebido (`tui-term`)

**Milestone:** M5 · **Estado:** CLOSED (target pendiente) · **Depende de:** M3-03, M4-01

## Objetivo
Ejecutar programas reales (shell, apps de pantalla completa) dentro del escritorio, y decidir con evidencia las dependencias de [ADR-0003](../adr/0003-terminal-embebido-pty-y-vt.md).

## Alcance (y qué NO entra)
- Crate `tui-term`: `Terminal` = `portable-pty` (PTY, hilo lector + canal) + `vt100` (pantalla) volcada a `Cell`/`Surface` (`draw`, `draw_with`), resize con SIGWINCH, codificación de teclas (flechas con DECCKM, F1-F12, Ctrl, Alt, modificadores xterm).
- `compat`: traduce HVP (`CSI f`) a CUP (`CSI H`), que `vt100` 0.16.2 no implementa.
- `tui-desktop`: sección `5 // TERMINAL`. Lanza `$SHELL` (TERM=xterm-256color) como el usuario de la sesión; con el foco en el contenido **todo** va al hijo salvo ALT+TAB (vuelve al menú). Al salir el hijo se vuelve a INICIO. Sondeo de 16 ms mientras hay terminal vivo (200 ms si no).
- `examples/spike.rs`: vuelca la pantalla de un programa antes y después de un resize.
- NO entra: el terminal como ventana del compositor (el escritorio sigue siendo de una vista, no usa `Compositor`; se decide en M6), grafemas (M5-02), ancho de emoji/CJK y Terminus (M5-03), ratón y pegado entre ventana y hijo, scrollback, respuestas a consultas del hijo (DSR/DA).

## Criterios de aceptación
- [x] Un shell corre en la sección, recibe teclas y su salida se dibuja
- [x] Resize del PTY llega al hijo (`stty size`)
- [x] Una app de curses de pantalla alternativa (`top`) y una de truecolor/Unicode (`btop`) se dibujan igual que en pyte (referencia)
- [x] Al salir el hijo no queda proceso huérfano ni bucle de relanzado
- [ ] `htop` y `vim`: no instalados en el host, sin probar

## Verificación
- VERIFIED (1-2): `scripts/test` (fmt, clippy `-D warnings`, 40+ tests) sin fallos; tests de PTY reales (`sh -c`), teclas, HVP y del flujo completo en `App`
- VERIFIED (spike): `top` y `btop` a 100×30 y tras resize a 70×20; btop contra pyte dio pantallas idénticas tras `compat`. btop a 70×20 muestra su propio aviso de tamaño mínimo (80×24), que es lo correcto
- VERIFIED: `yazi` se dibuja (sus iconos Nerd Font son de la deuda M5-03)
- VERIFIED (6, 2026-10-07): imagen reconstruida; en QEMU el terminal abre `sh` como `tui`, ejecuta fileTUI y ALT+TAB vuelve al menú (capturas en `build/shots/`)
- UNVERIFIED: `htop`, `vim` (ausentes); `cargo audit` (no instalado)

## Archivos cambiados
`crates/tui-term/**` (nuevo), `crates/tui-desktop/src/{app,main}.rs`, `crates/tui-desktop/Cargo.toml`, `Cargo.lock`, ADR-0003, roadmap.

## Limitaciones / deuda
- `vt100` 0.16.2 no implementa: HVP (parcheado en `compat`), REP (`CSI b`), respuestas a DSR/DA/consultas de color, ANSI save/restore (`CSI s/u`). **Si aparece un segundo hueco que haya que parchear, pasar a `vte` + modelo propio** (criterio del ADR).
- Solo se dibuja el primer `char` de cada celda (M5-02).
- Con el terminal enfocado Q/ESC/CTRL+Q no salen del escritorio: la salida es ALT+TAB y luego Q. ALT+TAB funciona en la consola de Linux del target (probado en QEMU).
- El hijo hereda el entorno del escritorio.
