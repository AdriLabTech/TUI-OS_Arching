# [M2-01] `tui-core`: Cell, Buffer, Surface

**Milestone:** M2 · **Estado:** CLOSED · **Depende de:** M0-02

## Objetivo
Primitivas del runtime TUI, en filas × columnas y sin dimensiones fijas.

## Alcance
`Cell`/`Style`/`Color`/`Attrs`, `Rect`, `Buffer` (set/get/fill/put_str/resize/blit/diff)
y `Surface` (Buffer + rectángulo de daño). Sin dependencias externas.
No entra: renderer (M2-02), entrada (M2-03), posición/z-order/foco (compositor, M3).

## Criterios de aceptación
- [x] Cualquier tamaño, incluido 0×0; fuera de rango nunca entra en pánico (recorta)
- [x] `diff` devuelve solo celdas cambiadas (y todas tras un resize)
- [x] `Surface` acumula daño y lo reinicia con `take_damage`; resize es un evento normal

## Verificación
- VERIFIED (nivel 1): `cargo fmt --check`, `clippy -D warnings`
- VERIFIED (nivel 2): 12 tests unitarios pasan
- FAILED y corregido: `Rect::union` truncaba 65 536 a 0 con `as u16`; ahora satura (test de bordes)
- NOT APPLICABLE: arranque en el target (es una librería; se verá con el renderer en M2-02)

## Limitaciones / deuda
- Un `char` = una celda: **sin soporte de ancho Unicode** (CJK, emoji, combinantes). Resolver antes de las apps (probablemente con `unicode-width`).
- `Buffer::diff` devuelve un `Vec`; si resulta caro, pasar a iterador (medir antes).

## Dependen de este
M2-02, M2-03, M3.
