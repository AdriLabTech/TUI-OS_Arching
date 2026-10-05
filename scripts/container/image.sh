#!/usr/bin/env bash
# Dentro del contenedor archlinux: rootfs.tar -> /out/tuios.img (GPT+ESP+ext4).
# Sin loop devices ni montajes: mke2fs -d y mtools pueblan los sistemas de ficheros.
# Solo toca ficheros de imagen bajo /tmp y /out (nunca dispositivos de bloque).
set -euo pipefail
SIZE_MIB=${IMAGE_SIZE_MIB:-4096}
ESP_MIB=512
ROOT_PARTUUID=5c0a4e5e-7a11-4c2b-9d3e-0b7f00000002   # fijo: reproducible
pacman -Sy --noconfirm --needed dosfstools mtools e2fsprogs util-linux mkinitcpio >/dev/null

R=/rootfs; W=/tmp/img; mkdir -p $R $W
tar --numeric-owner -C $R -xf /out/rootfs.tar
# Configuración propia de TUI-OS y binarios compilados en el host (misma glibc que el rootfs).
cp -a --no-preserve=ownership /work/config/overlay/. $R/
if [ -d /out/bin ]; then
  mkdir -p $R/usr/local/bin
  cp -a --no-preserve=ownership /out/bin/. $R/usr/local/bin/
  chmod 0755 $R/usr/local/bin/*
fi
KVER=$(ls $R/usr/lib/modules | head -n1)
echo "kernel: $KVER"

# initramfs (hooks por defecto de Arch; mkinitcpio -r usa el rootfs del target)
mkinitcpio -r $R -k "$KVER" -g $W/initramfs-linux.img >/dev/null

# Partición ESP
ESP_KIB=$((ESP_MIB*1024))
mkfs.fat -F32 -n TUIOSESP -C $W/esp.img $ESP_KIB >/dev/null
mmd -i $W/esp.img ::/EFI ::/EFI/BOOT ::/loader ::/loader/entries
mcopy -i $W/esp.img $R/usr/lib/systemd/boot/efi/systemd-bootx64.efi ::/EFI/BOOT/BOOTX64.EFI
mcopy -i $W/esp.img $R/usr/lib/modules/$KVER/vmlinuz ::/vmlinuz-linux
mcopy -i $W/esp.img $W/initramfs-linux.img ::/initramfs-linux.img
printf 'timeout 1\nconsole-mode keep\n' > $W/loader.conf
cat > $W/tuios.conf <<ENT
title   TUI-OS
linux   /vmlinuz-linux
initrd  /initramfs-linux.img
options root=PARTUUID=$ROOT_PARTUUID rw console=tty0 console=ttyS0,115200
ENT
mcopy -i $W/esp.img $W/loader.conf ::/loader/loader.conf
mcopy -i $W/esp.img $W/tuios.conf ::/loader/entries/tuios.conf

# Partición raíz
ROOT_START=$((2048+ESP_MIB*2048))
ROOT_SECTORS=$((SIZE_MIB*2048-ROOT_START-2048))   # deja 1 MiB para la GPT de respaldo
truncate -s $((ROOT_SECTORS*512)) $W/root.img   # exactamente el tamaño de la partición
mke2fs -q -t ext4 -L tuios-root -d $R -E root_owner=0:0 $W/root.img $((ROOT_SECTORS/2))K

# Ensamblado
rm -f /out/tuios.img
truncate -s ${SIZE_MIB}M /out/tuios.img
sfdisk -q /out/tuios.img <<PT
label: gpt
first-lba: 2048
start=2048, size=$((ESP_MIB*2048)), type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, name="esp"
start=$ROOT_START, size=$ROOT_SECTORS, type=4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709, uuid=$ROOT_PARTUUID, name="tuios-root"
PT
dd if=$W/esp.img  of=/out/tuios.img bs=512 seek=2048        conv=notrunc,sparse status=none
dd if=$W/root.img of=/out/tuios.img bs=512 seek=$ROOT_START conv=notrunc,sparse status=none
sfdisk -l /out/tuios.img | tail -4
