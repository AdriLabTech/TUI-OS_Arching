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
- M3-02 Ancho Unicode en `Cell`/`Buffer`/renderer (CJK, emoji; prerrequisito del terminal embebido) — CLOSED ([issue](issues/M3-02-ancho-unicode.md))
- M3-03 Fondo, recomposición solo si hay cambios, mosaico y ALT+TAB (corrige restos de M3-01) — CLOSED ([issue](issues/M3-03-mosaico-y-composicion.md))
- M3-04 Regiones sucias por rectángulo: solo si una medición lo justifica — PLANNED

## M4 — Sesión (antes del shell: no se abre una shell como root)
- M4-01 Escritorio como usuario `tui` (no root) — CLOSED ([issue](issues/M4-01-sesion-no-root.md))
- M4-02 Recuperación: tras 3 fallos el escritorio cae a un login en tty1 — CLOSED ([issue](issues/M4-02-recuperacion.md))

## M5 — Terminal embebido (`tui-term`)
- M5-01 Spike y [ADR-0003](adr/0003-terminal-embebido-pty-y-vt.md): `portable-pty` + `vt100`; shell, `top`, `btop` y `yazi` en la sección TERMINAL, con resize — CLOSED (target, `htop` y `vim` pendientes de verificar) ([issue](issues/M5-01-terminal-embebido.md))
- M5-02 Texto real en el terminal: grafemas (`unicode-segmentation`: combinantes y emoji ZWJ, hoy descartados o partidos) — PLANNED
- M5-03 Deuda de `unicode-width` heredada de M3-02 — PLANNED:
  - el ancho de emoji lo decide cada emulador, no el estándar: probar en el target y en un terminal real, con tabla de excepciones si hace falta;
  - la consola de Linux (Terminus) no tiene CJK ni emoji: glifo de reemplazo y documentarlo, o fuente con cobertura;
  - `cargo audit` sobre `unicode-width`, `unicode-segmentation`, `portable-pty` y `vt100`/`vte` al añadirlas.
- Prerrequisitos: M3-03 (mosaico) y M4 (sesión no-root).

## M6 — Apps de primera parte y comandos
- Herramientas propias en lugar de las de terceros (fileTUI, monitor, cliente Docker, cliente Git…): ver [apps.md](apps.md) y [ADR-0004](adr/0004-apps-de-primera-parte.md)
- Secciones del escritorio como apps/ventanas, convenciones de teclas (F1, ESC, CTRL+Q, ALT+TAB)
- `tui system|network|processes|storage|settings|install`, frontend de pacman

### Cambios respecto al plan anterior (2026-10-05)
- Arrastre de ventanas con ratón y bordes decorados: **descartados del camino crítico**. La consola de Linux no tiene ratón (deuda de M2-05) y el sistema es teclado primero; mosaico + ALT+TAB rinde más.
- Ancho Unicode sube a M3-02: cambia `Cell` y es mejor hacerlo antes de que dependan de él más apps. Sus limitaciones conocidas (grafemas, emoji, Terminus) se tratan en M5-02/03.
- Terminal embebido (antes ausente) pasa a ser el hito central: un compositor sin nada que componer salvo el escritorio actual no aporta; un terminal sí.
- Sesión (M4) antes del terminal: evita una shell root en tty1.
