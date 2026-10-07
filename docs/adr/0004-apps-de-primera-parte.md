# ADR-0004: Aplicaciones de primera parte

**Estado:** propuesta (se acepta al empezar la primera app integrada, M6)
**Contexto:** Adrián y Claude van a desarrollar las herramientas del sistema (fileTUI, un monitor tipo btop, un cliente de Docker tipo lazydocker, uno de Git tipo lazygit…; ver [apps.md](../apps.md)). fileTUI ya existe y usa Ratatui + Crossterm; TUI-OS tiene su propio runtime (`tui-core`, `tui-render`). Hay que decidir cómo se relacionan.

**Opciones:**
| Opción | A favor | En contra |
|---|---|---|
| A. Binarios independientes que corren en el terminal embebido (PTY) | fileTUI funciona ya, sin reescribir; cada app es utilizable fuera de TUI-OS; repos y ritmos propios | Cada app dibuja por su cuenta: la coherencia visual depende de compartir tema; no hay integración fina con el compositor |
| B. Reescribir las apps sobre `tui-core` como bibliotecas/secciones del escritorio | Integración total, un solo runtime y un solo tema | Reescribir fileTUI; acopla las apps a TUI-OS; tamaño del escritorio |
| C. A ahora y un SDK compartido después (crate de tema/widgets, agnóstico del backend) | Lo mejor de ambos si el SDK sale de código real, no de especulación | Requiere disciplina para no inventar el SDK antes de tener dos apps |

**Decisión propuesta:** A para empezar y C cuando existan al menos dos apps con duplicación real. B queda descartada salvo que una app necesite algo que el PTY no da. fileTUI sigue en su repo.

**Consecuencias:** el terminal embebido (M5) pasa a ser requisito de todas las apps: sus huecos (grafemas, fuente Terminus sin iconos, imágenes en el terminal) son huecos de las apps. fileTUI renderiza imágenes con protocolos gráficos del terminal (kitty u otros): **sin verificar** que `vt100` y la consola de Linux puedan soportarlo; hay que comprobarlo antes de prometer fileTUI con imágenes en TUI-OS. Si no es viable, esa parte queda para un modo gráfico futuro.
