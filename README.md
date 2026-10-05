# TUI-OS

Experimento: ¿cómo sería un sistema operativo moderno si un terminal/TUI fuese
la interfaz gráfica principal en lugar de un entorno de escritorio?

Linux (base Arch, systemd) es la infraestructura; TUI-OS es lo que se construye
encima: runtime TUI, compositor, aplicaciones y comandos de sistema en Rust.

> Estado: **Milestone 0 completo**; siguiente: M1 (target arrancable). Aún no arranca nada.

## Documentación
- [Arquitectura](docs/architecture.md)
- [Roadmap y milestones](docs/roadmap.md)
- [Plantilla de issue](docs/issues/TEMPLATE.md)
- [ADRs](docs/adr/)

## Entorno de desarrollo
Host: Arch Linux (x86_64) · QEMU/KVM + OVMF · Rust/Cargo · Docker (rootfs Arch).
El target (TUI-OS) nunca se construye tocando el host: todo va a imágenes en `build/`.

## Uso
```
./scripts/bootstrap   # comprueba el host (solo lectura)
./scripts/build       # compila el workspace
./scripts/test        # fmt + clippy + tests
```
