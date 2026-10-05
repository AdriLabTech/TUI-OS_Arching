# [M4-02] Recuperación del escritorio

**Milestone:** M4 · **Estado:** CLOSED · **Depende de:** M4-01

## Objetivo
Que un escritorio que falla no deje tty1 inutilizable.

## Alcance (y qué NO entra)
- `tui-desktop.service`: `StartLimitIntervalSec=30`, `StartLimitBurst=3` y `OnFailure=getty@tty1.service`. Tras 3 fallos en 30 s queda un login de texto en tty1.
- `scripts/image`: `TUIOS_DESKTOP_BIN` sustituye el binario del escritorio (para probar la recuperación).
- Se mantiene el nombre `tui-desktop.service`: renombrarlo a `tui-session` no aportaba nada. Autologin ya lo hacía el propio servicio desde M2-05.
- NO entra: reinicio del escritorio desde el login, modo seguro, diagnóstico en pantalla.

## Verificación
- VERIFIED (5): imagen con un escritorio que siempre sale con error → captura de tty1 con `tuios login:` (sin bucle de reinicios)
- VERIFIED (5): imagen normal reconstruida → captura del escritorio como `tui (uid 1000)`
- UNVERIFIED: que un pánico real (no `exit 1`) se comporte igual; la consola serie no se probó en este caso

## Limitaciones / deuda
- Tras la recuperación, volver al escritorio exige `systemctl start tui-desktop` como root o reiniciar.
- Root con contraseña de desarrollo sigue siendo la vía de recuperación.
