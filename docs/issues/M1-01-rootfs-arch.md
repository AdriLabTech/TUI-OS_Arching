# [M1-01] Rootfs Arch mínimo vía contenedor

**Milestone:** M1 · **Estado:** CLOSED · **Depende de:** M0-02

## Objetivo
`scripts/rootfs` produce `build/rootfs.tar` reproducible sin tocar el host.

## Alcance
Entra: `pacman -r` en un contenedor `archlinux` (sin `--privileged`, sin loop),
paquetes en `config/packages.txt` (`base linux mkinitcpio`), hostname, locale,
`os-release` de TUI-OS, getty en `ttyS0`, caché de paquetes en un volumen Docker.
No entra: initramfs, bootloader, imagen (M1-02).

## Criterios de aceptación
- [x] `scripts/rootfs` termina con exit 0 y genera `build/rootfs.tar`
- [x] El tar contiene kernel, `os-release` de TUI-OS y `/etc/shadow` con hash de root
- [x] No se usa root ni nada en el host

## Verificación
- VERIFIED: ejecución completa (~30 s con caché), tar de 588 MB
- VERIFIED: contenido inspeccionado (`vmlinuz` en `usr/lib/modules/7.2.8-arch1-2/`)
- UNVERIFIED: que el rootfs arranque (M1-03/M1-04)

## Limitaciones / deuda
- **Contraseña de root `tuios`: SOLO desarrollo.** Sustituir antes de cualquier uso real.
- `pacstrap` no se usa; los hooks de pacman que necesitan `/proc` fallan sin
  privilegios (se ven errores en la salida) y se rehacen a mano (hwdb aquí, initramfs en M1-02).
- `archlinux:latest` no está fijado por digest: la build no es bit a bit reproducible.

## Dependen de este
M1-02.
