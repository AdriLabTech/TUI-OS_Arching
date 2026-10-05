# [M1-02] Imagen de disco UEFI

**Milestone:** M1 · **Estado:** CLOSED · **Depende de:** M1-01

## Objetivo
`scripts/image` convierte `build/rootfs.tar` en `build/tuios.img` sin root en el host.

## Alcance
GPT con ESP FAT32 (512 MiB, systemd-boot, kernel e initramfs) y raíz ext4 con
PARTUUID fijo. Todo con `mke2fs -d`, `mtools` y `sfdisk` sobre ficheros, dentro de
un contenedor sin `--privileged`: no hay loop devices ni montajes. `IMAGE_SIZE_MIB`
cambia el tamaño (4096 por defecto).

## Criterios de aceptación
- [x] Imagen con tabla GPT válida y dos particiones (EFI System, Linux root x86-64)
- [x] Sin errores de GPT ni de tamaño
- [x] Arranca bajo OVMF (ver M1-04)

## Verificación
- VERIFIED: `sfdisk -l` correcto, fichero disperso (673 MB reales / 4,0 GB)
- FAILED y corregido: primera versión escribía `root.img` de 4 GiB y pisaba la GPT de respaldo
  (`PMBR size mismatch`). Ahora `root.img` mide exactamente la partición.

## Limitaciones / deuda
- Los avisos de `mkinitcpio` (firmware i915/nouveau/xe, `vconsole.conf`) son esperables en una VM.
- Se pierden los xattr/capabilities de ficheros (`mke2fs -d`); a revisar si hace falta (p. ej. `ping`).
- Sin Secure Boot.

## Dependen de este
M1-03, M1-04.
