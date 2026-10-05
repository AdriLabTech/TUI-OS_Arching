# Arquitectura

```
Application → TUI API → Surface/Buffer → Compositor → Renderer → Terminal/Framebuffer
```

- **Cell** (carácter, fg, bg, atributos) es la primitiva visual. La UI lógica
  trabaja en filas × columnas; nunca se asumen dimensiones fijas y el
  redimensionado es un evento normal.
- **Linux es infraestructura**: kernel, drivers, red, FS, systemd y pacman se
  reutilizan. Las apps hablan con una API de TUI-OS, no con comandos sueltos.
- **Stack**: x86_64, UEFI, Arch Linux, systemd, Rust (workspace Cargo).
- **Sin escritorio gráfico** (GNOME/KDE/XFCE/GTK/Qt) salvo issue explícito.
- **Recuperación**: siempre hay consola serie y shell de depuración en desarrollo.

## Host / Target
| | Qué es |
|---|---|
| HOST | Máquina de desarrollo (Arch). No se modifica fuera del repo. |
| TARGET | Imagen TUI-OS arrancada en QEMU/KVM. |

El rootfs Arch se construye dentro de un contenedor (`pacstrap`), sin instalar
nada de arranque en el host. Ver [ADR-0002](adr/0002-rootfs-en-contenedor.md).
