# Informe de cierre — Milestone 2: Runtime TUI

**Estado:** completo en el HOST; verificación en el TARGET pendiente (no existe aún una sesión TUI).

| Issue | Resultado |
|---|---|
| M2-01 `tui-core` | CLOSED, 12 tests |
| M2-02 renderer | CLOSED, host verificado, target pendiente |
| M2-03 entrada | CLOSED, host verificado, target pendiente |

**Suite:** `scripts/test` (fmt + clippy `-D warnings` + 25 tests) en verde. **Demo:** `cargo run --example hello -p tui-render`.

**Regresión M1:** `scripts/boot-test` no se repitió tras M2 (M2 no toca la imagen).

## Deuda técnica
1. Sin ancho Unicode (CJK, emoji, combinantes) — bloquea apps con texto real.
2. `Renderer` clona el `Buffer` por fotograma; `Buffer::diff` asigna un `Vec`. Medir antes de optimizar.
3. Verificación en el target: falta un binario TUI en la imagen y una sesión que lo lance (M4).
4. Entrada: sin protocolo de teclado kitty ni pegado entre corchetes.

## Bloqueos para M3 (compositor)
Ninguno técnico. Conviene decidir antes el ancho Unicode, porque cambia `Cell` y el recorte de superficies.

## Adenda (mismo día): escritorio y arranque en QEMU
Se adelantaron M2-04 (escritorio TUI) y M2-05 (arranque en QEMU, adelanto de M4), a petición del autor, para tener algo visible. La deuda 3 (verificación en el target) queda **cerrada para el renderer, la paleta y el escritorio**; sigue abierta la parte de sesión de usuario no-root, y el ancho Unicode. Suite actual: 51 tests.
