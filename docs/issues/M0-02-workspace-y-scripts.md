# [M0-02] Workspace Cargo y scripts de desarrollo

**Milestone:** M0 · **Estado:** CLOSED · **Depende de:** M0-01

## Objetivo
Workspace Rust y scripts predecibles que comprueben el host sin modificarlo.

## Alcance (y qué NO entra)
Entra: workspace, crate `tui-core` vacío, `scripts/{bootstrap,build,test,run,clean}`.
No entra: imagen del target (M1), primitivas TUI (M2). `run` falla con mensaje claro hasta M1-03.

## Criterios de aceptación
- [x] `scripts/bootstrap` informa del host y falla si falta algo imprescindible
- [x] `scripts/test` ejecuta fmt, clippy (`-D warnings`) y tests
- [x] `scripts/build` compila el workspace
- [x] Ningún script modifica el host

## Verificación
- VERIFIED (nivel 1): `cargo fmt --check` y `cargo clippy -D warnings` sin avisos (vía `scripts/test`)
- VERIFIED (nivel 2): `cargo test`: 1 passed
- VERIFIED: `scripts/bootstrap` exit 0 en Arch x86_64, KVM y OVMF presentes; avisa de Docker apagado
- VERIFIED: `scripts/build` exit 0; `scripts/run` exit 1 con mensaje esperado
- UNVERIFIED: `scripts/clean` (no ejecutado)
- NOT APPLICABLE: arranque del target (aún no existe)

## Archivos
`Cargo.toml`, `Cargo.lock`, `crates/tui-core/`, `scripts/*`

## Limitaciones / deuda
`bootstrap` solo comprueba; la instalación de dependencias es manual (Arch: `pacman`).
Rutas de OVMF probadas: 3 ubicaciones conocidas.

## Dependen de este
M1-01 (Docker), M1-03 (`scripts/run`), M2-01 (`tui-core`).
