# [M3-02] Ancho Unicode en Cell/Buffer/renderer

**Milestone:** M3 · **Estado:** CLOSED · **Depende de:** M3-01

## Objetivo
Que CJK y emoji ocupen 2 columnas sin descuadrar el renderer; prerrequisito del terminal embebido (M5), que recibirá texto arbitrario.

## Alcance (y qué NO entra)
- Dependencia nueva: `unicode-width` 0.2 (tablas del estándar; no se reimplementan a mano).
- `Cell::CONT` marca la 2.ª mitad de un carácter ancho; `Cell::width()`.
- `Buffer::put_str` avanza por columnas; el ancho que no cabe en el borde se sustituye por un espacio; los de ancho 0 se descartan.
- `Buffer::set` y `blit` nunca dejan media celda huérfana (pisado o recorte).
- El renderer omite las continuaciones y avanza el cursor por el ancho.
- NO entra: clústeres de grafemas (emoji ZWJ, combinantes), la consola de Linux (Terminus no tiene CJK).

## Verificación
- VERIFIED (1-2): `scripts/test`, 62 tests (6 nuevos: 5 en tui-core, 1 en tui-render)
- UNVERIFIED: aspecto en un terminal real con fuente CJK; la consola de Linux pinta glifos de reemplazo

## Limitaciones / deuda
- Sin grafemas: `e` + U+0301 pierde el acento. Considerar `unicode-segmentation` en M5.
- El ancho de emoji depende del terminal; `unicode-width` sigue el estándar, no cada emulador.
