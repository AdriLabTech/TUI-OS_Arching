# ADR-0002: El rootfs Arch se construye en un contenedor

**Estado:** aceptada
**Contexto:** el host es Arch pero sin `arch-install-scripts`; instalar
herramientas de bootstrap con root en el host rompe la separación HOST/TARGET.
**Decisión:** construir el rootfs con `pacstrap` dentro de un contenedor Docker
(`archlinux`) y empaquetarlo en una imagen de disco UEFI bajo `build/`.
**Consecuencia:** reproducible y sin tocar el host; requiere el daemon de Docker.
