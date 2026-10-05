#!/usr/bin/env bash
# Se ejecuta DENTRO del contenedor archlinux. Construye el rootfs del TARGET
# en /rootfs y lo exporta como /out/rootfs.tar. Sin privilegios especiales.
set -euo pipefail
PKGLIST=/work/config/packages.txt
mapfile -t PKGS < <(grep -vE '^\s*(#|$)' "$PKGLIST")

pacman-key --init >/dev/null 2>&1
pacman-key --populate archlinux >/dev/null 2>&1
mkdir -p /rootfs/var/lib/pacman
# --noscriptlet: no hay /proc ni /dev en un chroot sin privilegios; lo que
# hacían (initramfs, units) lo hacemos explícitamente más abajo.
pacman -Sy --noconfirm --noscriptlet --needed -r /rootfs --cachedir /var/cache/pacman/pkg "${PKGS[@]}"

R=/rootfs
# Los hooks de pacman que necesitan /proc o /dev fallan aquí; se rehacen a mano.
systemd-hwdb --root=$R update
echo tuios > $R/etc/hostname
echo 'LANG=en_US.UTF-8' > $R/etc/locale.conf
sed -i 's/^#en_US.UTF-8/en_US.UTF-8/' $R/etc/locale.gen
ln -sf /usr/share/zoneinfo/UTC $R/etc/localtime
# Contraseña de desarrollo (SOLO dev; ver docs/issues/M1-01). Hash generado aquí.
echo 'root:tuios' | chpasswd -R $R
# Servicios mínimos: arranque por defecto en multi-user con getty en la serie.
ln -sf /usr/lib/systemd/system/multi-user.target $R/etc/systemd/system/default.target
mkdir -p $R/etc/systemd/system/getty.target.wants
ln -sf /usr/lib/systemd/system/serial-getty@.service $R/etc/systemd/system/getty.target.wants/serial-getty@ttyS0.service
# Identidad del sistema
cat > $R/etc/os-release <<'OSR'
NAME="TUI-OS"
PRETTY_NAME="TUI-OS (Arch base)"
ID=tuios
ID_LIKE=arch
OSR
tar --numeric-owner -C $R -cf /out/rootfs.tar .
echo "rootfs.tar: $(du -h /out/rootfs.tar | cut -f1)"
