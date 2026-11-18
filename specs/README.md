# Índice de especificaciones de SIPI-brain

| Spec | Rama de implementación | Integración local | Evidencia de entrega |
| --- | --- | --- | --- |
| [001 — Memoria colectiva](001-sipibrain-collective-memory/spec.md) | `codex/sipibrain-mvp` | Spec en `main`; el incremento del MVP aún no está integrado. | [Tareas y comprobaciones pendientes](001-sipibrain-collective-memory/tasks.md). |
| [002 — Creación y supervisión de LLMwikis](002-wiki-admin-monitoring/spec.md) | `codex/002-wiki-admin-monitoring` | No integrada en `main`. | [Tareas y evidencia](002-wiki-admin-monitoring/tasks.md). |
| [003 — Alta de agentes y prueba de conexión](003-admin-agent-onboarding/spec.md) | `codex/003-admin-agent-onboarding` | No integrada en `main`; alcance local verificado con datos sintéticos. | [SC-004 y T025 pendientes](003-admin-agent-onboarding/quickstart.md#aceptación-real-sc-004): falta verificar la identidad con un agente SIPI real autorizado y acreditar el cierre constitucional. |

La presencia de una especificación en Git no acredita la publicación de su entrega.

## Relaciones entre las especificaciones

- **001 → 002:** La 001 define una wiki por agente, las asignaciones y el intercambio mediado. La 002 añade creación y supervisión desde el panel; el rol administrativo no sustituye la asignación necesaria para leer contenido.
- **002 → 003:** La 003 reutiliza el alta de wikis de la 002, añade el registro de agentes y separa la creación de la wiki de la configuración y la prueba de conexión.
- **001 y 003:** La prueba de la 003 verifica identidad, no concede acceso ni demuestra intercambio de conocimiento. La verificación del intercambio real autorizado sigue en la 001 y su evidencia se requiere para el cierre constitucional de la 003.
