# [M2-04] Escritorio TUI con identidad propia

**Milestone:** M2 (adelantado respecto al roadmap original, que lo situaba en M5) · **Estado:** CLOSED · **Depende de:** M2-03

## Objetivo
Una interfaz TUI usable y reconocible, no solo una demo de eventos.

## Alcance
- Renderer: paleta de 16 colores reprogramable (`OSC P`) cuando `TERM=linux`; en terminales con truecolor se usan los `Rgb` tal cual. En modo paleta se omiten negrita/tenue (en fbcon cambian el índice de color).
- Crate `tui-desktop`: barra superior (marca, equipo, CPU/MEM, hora UTC), menú numerado, contenido con título `NN // SECCIÓN`, panel «Resumen» (≥110 columnas) y barra de atajos. Secciones: Inicio (logo, ficha técnica, disco-sol, tarjetas), Sistema (CPU histórico y por núcleo, memoria, swap), Procesos (tabla desde `/proc`), Archivos (navegador de directorios), Ayuda.
- Tema «papel y tinta» (guía de estilo del autor): negro, hueso `#cdc4ba`, rampa de 4 tonos, bermellón y oro solo como acento, sin sombras ni degradados.
- Lectura de Linux aislada en `sys.rs` (parsers puros sobre `&str`); `--snapshot WxH [sección]` vuelca un fotograma como texto.
- Teclas: F1 ayuda, 0-4 secciones, ↑↓, Enter/→, Tab/Esc (foco), RePág/AvPág, Backspace/← (subir), Q/Ctrl+Q. Ratón: clic y rueda (en emuladores de terminal).

## Criterios de aceptación
- [x] Se dibuja bien entre 48×12 y 300×80 (mensaje claro por debajo del mínimo); sin pánicos
- [x] Todos los glifos existen en la fuente de consola (test de regresión)
- [x] Los colores del tema salen exactos en la consola de Linux

## Verificación
- VERIFIED (1-2): fmt, clippy `-D warnings`, 51 tests (14 core, 22 desktop, 15 render)
- VERIFIED (3): volcados de texto a 80×24, 100×28, 120×36
- VERIFIED (4-5, en el TARGET): capturas de la consola de la VM (1280×800): píxeles medidos `#e0563b` (borde con foco), `#3a3733` (sin foco), `#e2342a` (marca), `#1b1b1b` (disco) = tokens del tema
- FAILED y corregido: la fuente Terminus no tiene `▀ ▄ ▌ ▐ ▓` ni `▸` (salían como `#`); se leyó su tabla Unicode, se sustituyeron y se añadió el test de glifos
- FAILED y corregido: disco-sol enorme y con puntas; ahora ocupa la zona libre sobre las tarjetas
- UNVERIFIED: ratón en la consola de Linux (no hay `gpm`; el modo de ratón de crossterm no aplica a fbcon); varios núcleos (la VM tiene 2)

## Limitaciones / deuda
- Un `char` = una celda (sin ancho Unicode): no hay kanji (力) ni emoji.
- Los widgets (`ui.rs`) y la capa `sys.rs` son locales a la app; promover a crates cuando exista el compositor (M3).
- El gráfico de área tiene resolución de filas completas (sin medios bloques por la fuente).
- Hora en UTC fija; sin zona horaria.

## Dependen de este
M3 (compositor), M5 (apps de primera parte).
