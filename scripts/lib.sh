# Utilidades comunes. Se carga con `. scripts/lib.sh`.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT/build"
need() { command -v "$1" >/dev/null 2>&1; }
