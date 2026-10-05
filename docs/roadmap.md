# Roadmap

Estados: PLANNED → READY → IN PROGRESS → IMPLEMENTED → VERIFIED → DOCUMENTED → COMMITTED → CLOSED.

## M0 — Cimientos
- M0-01 Repo, docs, ADRs, roadmap — CLOSED
- M0-02 Workspace Cargo + `scripts/` (`bootstrap`, `build`, `test`, `run`, `clean`) con comprobación del host — CLOSED ([issue](issues/M0-02-workspace-y-scripts.md))

## M1 — Target arrancable
- M1-01 Rootfs Arch mínimo vía contenedor (`pacstrap`) — CLOSED
- M1-02 Imagen de disco UEFI (GPT + ESP + systemd-boot) en `build/` — CLOSED
- M1-03 `scripts/run` en QEMU/KVM + OVMF, consola serie a fichero de log — CLOSED
- M1-04 Prueba de arranque automatizada (espera `login:` por serie) — CLOSED

## M2 — Runtime TUI
- M2-01 `tui-core`: `Cell`, `Buffer`, `Surface` (tests unitarios) — CLOSED
- M2-02 Renderer a terminal con detección de tamaño y evento de resize — CLOSED (target pendiente)
- M2-03 Entrada: teclado y ratón como eventos — CLOSED (target pendiente) · [informe M2](milestones/M2-informe.md)

- M2-04 Escritorio TUI con identidad propia (adelantado de M5) — CLOSED ([issue](issues/M2-04-escritorio-tui.md))
- M2-05 El escritorio arranca solo en QEMU (adelanto de M4) — CLOSED ([issue](issues/M2-05-escritorio-en-qemu.md))

## M3 — Compositor
- M3-01 Superficies, z-order, foco, recorte y enrutado de eventos — CLOSED ([issue](issues/M3-01-compositor.md))
- M3-02 Regiones sucias en la composición; decoración y arrastre de ventanas — PLANNED

## M4 — Sesión
- Servicio systemd `tui-session`, autologin a la TUI, entorno de recuperación

## M5 — Apps de primera parte
- `tui-system`, `tui-files`, `tui-processes`, convenciones de teclas (F1, ESC, CTRL+Q, ALT+TAB)

## M6 — Comandos y paquetes
- `tui system|network|processes|storage|settings|install`, frontend de pacman
