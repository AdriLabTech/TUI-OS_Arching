# [M2-05] El escritorio arranca solo en QEMU

**Milestone:** M2 (adelanto de M4: sesión mínima) · **Estado:** CLOSED · **Depende de:** M2-04, M1-04

## Objetivo
Ver y probar TUI-OS en QEMU: arranque → escritorio TUI en la consola gráfica (tty1).

## Alcance
- `config/overlay/` se copia sobre el rootfs al crear la imagen: `tui-desktop.service` (tty1, `Restart=on-failure`; al salir con Q lanza `getty@tty1`), `vconsole.conf` (`KEYMAP=es`, `FONT=ter-v16n`) y el enlace de activación.
- `terminus-font` en `config/packages.txt`.
- `scripts/image` compila `tui-desktop` en release (glibc idéntica a la del rootfs: 2.44) y lo instala en `/usr/local/bin`.
- `scripts/run`: `MONITOR=<socket>` para el monitor de QEMU. `scripts/screenshot`: arranca sin ventana, envía teclas y guarda PNG. `scripts/demo`: ventana gtk.
- Recuperación: consola serie (`ttyS0`) intacta, Ctrl+Alt+F2 abre otra consola, y Q devuelve un login en tty1.

## Verificación
- VERIFIED (4): `scripts/boot-test` sigue llegando a `tuios login:` por la serie (~11 s)
- VERIFIED (5): capturas de la VM de Inicio, Sistema, Procesos (con `tui-desktop` y `agetty` en la lista), Archivos (entra en `/root/.ssh`) y Ayuda, navegando solo con `sendkey`
- VERIFIED (5): Q → `tuios login:` en tty1 con la paleta restaurada
- VERIFIED: `scripts/demo` abre QEMU con ventana gtk bajo Wayland sin errores (arranca hasta el login; cerrado por `timeout`)
- UNVERIFIED: apariencia dentro de la ventana gtk vista por una persona (se verificó con `screendump`, no con los ojos); Ctrl+Alt+F2; hardware real

## Limitaciones / deuda
- Contraseña de root `tuios` (solo desarrollo).
- Sin ratón en la consola de Linux.
- `tui-desktop` corre como root en tty1: pasar a un usuario con sesión propia (M4).
- Las capturas dependen de un tiempo de espera fijo (`WAIT=14`), no de una señal del escritorio.
