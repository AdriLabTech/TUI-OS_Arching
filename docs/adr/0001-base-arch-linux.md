# ADR-0001: Base Arch Linux + systemd + Rust

**Estado:** aceptada
**Decisión:** reutilizar Linux (Arch, systemd, pacman) como infraestructura y
escribir en Rust el userspace TUI.
**Motivo:** rolling release, rootfs mínimo con `pacstrap`, no reinventar kernel,
drivers ni gestor de paquetes. Rust por tipado fuerte y ecosistema TUI.
**Consecuencia:** TUI-OS es un proyecto de integración y arquitectura.
