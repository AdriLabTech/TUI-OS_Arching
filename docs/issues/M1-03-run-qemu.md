# [M1-03] `scripts/run` en QEMU/KVM

**Milestone:** M1 · **Estado:** CLOSED · **Depende de:** M1-02

## Objetivo
Arrancar el TARGET de forma repetible, observable y sin tocar el host.

## Alcance
QEMU q35 con KVM (cae a TCG con aviso), OVMF con copia propia de VARS en `build/`,
disco virtio, red user-mode y consola serie en la terminal (salir: `Ctrl-A X`).
Variables: `LOG`, `MEM`, `CPUS`, `DISPLAY_MODE`, `SNAPSHOT`. Por defecto el disco
va en modo snapshot: los cambios no persisten (`SNAPSHOT=0` para persistir).

## Criterios de aceptación
- [x] Arranca con UEFI y deja la serie en una terminal
- [x] Registra la serie en un fichero si se pide (`LOG`)

## Verificación
- VERIFIED: arranque headless con `LOG` (usado por M1-04)
- UNVERIFIED: modo interactivo en terminal (`Ctrl-A X`); ventana gráfica (`DISPLAY_MODE`)

## Limitaciones / deuda
Sin SSH ni puerto de depuración todavía; la consola serie es el único acceso.

## Dependen de este
M1-04, todos los issues con verificación en runtime.
