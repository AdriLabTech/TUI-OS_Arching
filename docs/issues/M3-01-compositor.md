# [M3-01] Compositor: ventanas, z-order, foco, recorte y enrutado

**Milestone:** M3 · **Estado:** CLOSED · **Depende de:** M2-01, M2-03

## Objetivo
Que varias superficies convivan en pantalla con posición, orden y foco propios.

## Alcance (y qué NO entra)
- `tui_core::Compositor`: `add`, `remove`, `move_to`, `raise`, `focused`, `window_at`, `compose`, `route`, `visible_rect`.
- Posición `i32` (ventanas parcial o totalmente fuera de pantalla); el recorte lo hace `Buffer::blit`.
- Foco = ventana superior. Teclado → foco. Ratón → ventana bajo el cursor, con coordenadas locales; un clic la sube.
- NO entra: regiones sucias en la composición (compone todo cada vez), bordes/títulos de ventana, arrastre, ALT+TAB, migrar `tui-desktop` al compositor, ancho Unicode.

## Criterios de aceptación
- [x] La ventana superior gana en las zonas solapadas; `raise` cambia el orden y el foco
- [x] Ventanas fuera de pantalla no provocan pánico y `visible_rect` devuelve el recorte
- [x] Teclas a la ventana con foco; ratón a la superior con coordenadas locales; clic sube y enfoca
- [x] Al quitar la superior el foco pasa a la siguiente; los ids no se reutilizan

## Verificación
- VERIFIED (1-2): `scripts/test` (fmt + clippy `-D warnings` + 56 tests, 5 nuevos en `compositor.rs`)
- UNVERIFIED: uso real desde una app; rendimiento (composición completa por fotograma)

## Archivos cambiados
`crates/tui-core/src/compositor.rs` (nuevo), `crates/tui-core/src/lib.rs`

## Limitaciones / deuda
- Sin regiones sucias: el siguiente issue puede usar `Surface::take_damage` (hoy se descarta).
- `Event::Resize` no se enruta; lo gestiona el dueño del compositor.

## Issues que dependen de este
M3-02 (decoración/arrastre de ventanas y regiones sucias), M5 (apps de primera parte)
