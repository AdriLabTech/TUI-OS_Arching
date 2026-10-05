# [M4-01] Sesión no-root

**Milestone:** M4 · **Estado:** CLOSED · **Depende de:** M2-05

## Objetivo
Que el escritorio no corra como root, requisito previo al terminal embebido (M5).

## Alcance (y qué NO entra)
- Usuario `tui` (uid 1000, `nologin`, sin contraseña) creado en `scripts/container/image.sh`; `tui-desktop.service` con `User=tui` y `HOME=/home/tui`.
- El escritorio muestra `USUARIO` (Inicio y Sistema), leído de `/proc/self/status` y `/etc/passwd`.
- Root sigue con contraseña de desarrollo en la consola serie (recuperación).
- NO entra: sesión con login real, entorno de recuperación propio (M4-02).

## Verificación
- VERIFIED (1-2): `scripts/test`, fmt + clippy + 68 tests (1 nuevo, parseo de uid/passwd)
- VERIFIED (4): `scripts/boot-test` llega a `tuios login:` (~11 s)
- VERIFIED (5): captura de la VM: `USUARIO tui (uid 1000)`, carpeta `/home/tui`, 121 procesos
- UNVERIFIED: que Q devuelva el login en tty1 sin root; ALT+TAB en la consola de Linux

## Limitaciones / deuda
- Contraseña de root `tuios` (solo desarrollo) sigue activa.
