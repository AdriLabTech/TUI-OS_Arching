# Roadmap

Estados: PLANNED → READY → IN PROGRESS → IMPLEMENTED → VERIFIED → DOCUMENTED → COMMITTED → CLOSED.

## M0 — Cimientos
- M0-01 Repo, docs, ADRs, roadmap — CLOSED
- M0-02 Workspace Cargo + `scripts/` (`bootstrap`, `build`, `test`, `run`, `clean`) con comprobación del host — CLOSED ([issue](issues/M0-02-workspace-y-scripts.md))

## M1 — Target arrancable
- M1-01 Rootfs Arch mínimo vía contenedor (`pacstrap`)
- M1-02 Imagen de disco UEFI (GPT + ESP + systemd-boot) en `build/`
- M1-03 `scripts/run` en QEMU/KVM + OVMF, consola serie a fichero de log
- M1-04 Prueba de arranque automatizada (espera `login:` por serie)

## M2 — Runtime TUI
- M2-01 `tui-core`: `Cell`, `Buffer`, `Surface` (tests unitarios)
- M2-02 Renderer a terminal con detección de tamaño y evento de resize
- M2-03 Entrada: teclado y ratón como eventos

## M3 — Compositor
- Superficies, z-order, foco, recorte, regiones sucias, enrutado de eventos

## M4 — Sesión
- Servicio systemd `tui-session`, autologin a la TUI, entorno de recuperación

## M5 — Apps de primera parte
- `tui-system`, `tui-files`, `tui-processes`, convenciones de teclas (F1, ESC, CTRL+Q, ALT+TAB)

## M6 — Comandos y paquetes
- `tui system|network|processes|storage|settings|install`, frontend de pacman
