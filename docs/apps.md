# Herramientas propias de TUI-OS

Principio: **el sistema no instala herramientas de terceros donde pueda haber una nuestra**. Las herramientas de primera parte las desarrollan Adrián y Claude juntos, en Rust, con la estética del sistema (papel y tinta, ver README). Las de terceros son un puente hasta que exista la nuestra.

| Herramienta | Sustituye a | Estado | Repo / ubicación | Notas |
|---|---|---|---|---|
| **fileTUI** | yazi (y `mc`, `ranger`) | Existe: base terminada, visor de imágenes en curso | `~/Proyectos/fileTUI` (Rust, Ratatui, Crossterm) | Será el gestor de archivos por defecto. Probado en el terminal embebido (host): funciona, salvo las imágenes (solo kitty/WezTerm; ver ADR-0004). Pendiente: render de imágenes más rápido, PDF. |
| Monitor de sistema | btop | Idea | sin repo | Procesos, CPU, memoria, disco, red. Hoy el escritorio ya tiene SISTEMA y PROCESOS básicos como germen. |
| Cliente de Docker | lazydocker | Idea | sin repo | Contenedores, imágenes, logs, volúmenes. |
| Cliente de Git | lazygit | Idea | sin repo | Estado, staging por hunks, ramas, log, stash. |
| Más herramientas | — | Por definir | — | Se añaden aquí según surjan (editor, frontend de pacman, red, ajustes: ver M6 del roadmap). |

Los nombres de las herramientas nuevas están **sin decidir**.

## Cómo se integran (ADR-0004, aceptado)
Ver [ADR-0004](adr/0004-apps-de-primera-parte.md). Resumen: cada herramienta es un **binario independiente** que se ejecuta en el terminal embebido (`tui-term`), con su propio repo y ciclo de vida. El escritorio las lanza desde el menú. Así fileTUI funciona hoy sin tocarlo.

## Reglas
- Una herramienta propia sustituye a la de terceros en `config/packages.txt` solo cuando cubre lo que se usa a diario.
- Mientras tanto, la de terceros puede venir instalada y marcada como provisional en la tabla.
- Cada herramienta nueva: nota en el vault (`04 - Projects`), fila aquí y entrada en el índice de repositorios.
