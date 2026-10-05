# [M1-04] Prueba de arranque automatizada

**Milestone:** M1 · **Estado:** CLOSED · **Depende de:** M1-03

## Objetivo
`scripts/boot-test` arranca el TARGET y exige el prompt de login por la serie.

## Criterios de aceptación
- [x] Exit 0 si aparece `tuios login:` antes de `TIMEOUT` (90 s)
- [x] Exit 1 con las últimas líneas del log si no aparece o QEMU termina
- [x] No deja QEMU huérfano

## Verificación (evidencia)
- VERIFIED: `OK: login alcanzado en ~12s`; `build/boot.log` contiene
  `Welcome to TUI-OS (Arch base)!` y `TUI-OS (Arch base) 7.2.8-arch1-2 (ttyS0)` / `tuios login:`
- VERIFIED: 0 coincidencias de `kernel panic` o `failed to start` en el log
- VERIFIED (negativa): con `TIMEOUT=2` sale con 1; `pgrep qemu` vacío después
- UNVERIFIED: inicio de sesión real como root (contraseña de desarrollo) y red

## Limitaciones / deuda
Solo comprueba que llega al login, no la salud de los servicios.
