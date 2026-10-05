# [M3-03] Composición con fondo, mosaico y ALT+TAB

**Milestone:** M3 · **Estado:** CLOSED · **Depende de:** M3-01, M3-02

## Objetivo
Que el compositor sea usable con teclado: sin restos al mover/cerrar ventanas, disposición automática y cambio de foco.

## Alcance (y qué NO entra)
- **Corrección de M3-01:** `compose` no pintaba fondo y dejaba restos de ventanas movidas o cerradas. Ahora rellena con `set_background` y recompone.
- `compose` devuelve `false` y no toca `out` si nada cambió (`needs_compose`: cambio estructural, tamaño de salida o daño en alguna superficie).
- Foco independiente del z-order (`focus`, `set_focus`, `cycle_focus`); `route` consume ALT+TAB y ALT+SHIFT+TAB (`Routed::Consumed`).
- `Layout::Tiled`: maestra a la izquierda (60 %) y resto apiladas a la derecha; recoloca y redimensiona superficies según el tamaño de salida. `Layout::Floating` mantiene M3-01.
- NO entra: regiones sucias por rectángulo (se omite la recomposición completa; el renderer ya envía solo celdas distintas). Medir antes de añadirlas. Tampoco decoraciones ni arrastre (ver roadmap).

## Verificación
- VERIFIED (1-2): `scripts/test`, 67 tests (5 nuevos de composición/mosaico/foco; los de M3-01 pasan sin cambios)
- Propiedad comprobada: para 1-5 ventanas y tamaños 80×24 y 7×5, cada celda pertenece exactamente a una ventana (sin huecos ni solapes)
- UNVERIFIED: coste de recomponer a pantalla completa (no medido); uso desde una app real

## Limitaciones / deuda
- Con anchura de pantalla de 1 columna las ventanas de la pila quedan con ancho 0.
- ALT+TAB depende de que el terminal envíe ALT como modificador; la consola de Linux lo hace como prefijo ESC (comprobar en M4/M6).
- Sin layouts configurables (ratio, orientación).
