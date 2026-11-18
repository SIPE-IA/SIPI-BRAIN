# Plan de implementación: alta de agentes y prueba de conexión

**Rama de la especificación:** `codex/003-admin-agent-onboarding` | **Fecha:** 2026-11-18 | **Especificación:** [003](spec.md)
**Contexto local:** Spec Kit resolvió `003-admin-agent-onboarding` desde `.specify/feature.json`.

## Resumen

Ampliar el panel administrativo para registrar el primer agente, crearle una wiki reutilizando el flujo 002 y verificar su identidad contra un destino compatible preaprobado. Alta, wiki y prueba son independientes. La prueba no concede permisos, no usa contenido y no se confunde con el intercambio de la 001.

## Contexto técnico

| Aspecto | Decisión |
| --- | --- |
| Lenguaje y dependencias | Rust 1.98 (edición 2024), Axum 0.8, Tokio, SQLx/SQLite; Vue 3.5, TypeScript 5.9 y Vite 8 según manifiestos locales. Reutilizar las dependencias y guardas existentes; el transporte real depende del contrato aprobado. |
| Persistencia | Tablas existentes para agente y wiki; una configuración opcional por agente. Resultados de prueba efímeros. Catálogo autoritativo del entorno, versionado y activado de forma atómica fuera del panel, en el backend en ejecución y sin reinicio; la lectura al arrancar no basta. La activación del catálogo, el inicio del contacto y la confirmación del resultado tienen un orden inequívoco. Si no se puede comprobar la aprobación vigente, no se inicia el contacto ni se confirma el éxito. |
| Pruebas y plataforma | Aplicación web local del piloto. Integración Rust con SQLite temporal y doble local controlado; pruebas de la vista con Vitest y Vue Test Utils; aceptación separada desde el panel con un agente SIPI real autorizado. |
| Límites | 10 s totales iniciales (hasta 3 s de conexión si el transporte lo permite); sin reintentos automáticos ni redirecciones. No se fija un SLO de negocio no solicitado. |
| Alcance | Registro, reutilización de alta de wiki, configuración y prueba. Sin aprobación de destinos desde el panel, descubrimiento automático, credenciales nuevas ni intercambio. |

## Verificación constitucional

| Principio | Evaluación previa | Reevaluación tras el diseño |
| --- | --- | --- |
| 1. Simplicidad y alcance | Reutilizar la 002; separar las tres historias. | Conforme: sin editor de destinos, nueva bóveda ni historial de pruebas. |
| 2. Memoria basada en LLM Wiki | Preservar los flujos de contenido existentes. | Conforme: no se modifican fuentes, síntesis, índice ni historial. |
| 3. Aislamiento e intercambio mediado | La administración no otorga lectura. | Conforme: no se tocan asignaciones ni se invoca el mediador; una prueba exitosa no autoriza intercambio. |
| 4. Trazabilidad verificable | Correlacionar la operación con el agente y la configuración, sin contenidos. | Conforme en diseño: revisión y estado acotados; evidencia no sensible en la validación real. |
| 5. Contexto, no autoridad | No tratar memoria, disponibilidad o configuración como permiso. | Conforme: el servidor verifica aprobación vigente e identidad por separado. |
| 6. Datos mínimos y secretos | No exponer ni persistir credenciales en las entidades de la 003. | Conforme: clave opaca, identidad y revisión; el canal autorizado permanece fuera del panel. |
| Verificación | Casos observables y prueba real no sustituible. | **Verificación de entrega pendiente.** Además de SC-001 a SC-004, el cierre exige evidencia aplicable al candidato de persistencia, recuperación, separación, trazabilidad e intercambio permitido y denegado. SC-004 acredita identidad, no intercambio. Si falta evidencia constitucional, informar el resultado como parcial y no cerrar la entrega. Ver [evidencia de cierre](quickstart.md#evidencia-de-cierre-constitucional). |

## Estructura del proyecto

- Diseño: [investigación](research.md), [modelo de datos](data-model.md), [contrato API/panel](contracts/agent-onboarding.md), [guía de validación](quickstart.md).
- Backend: `backend/src/api.rs`, `backend/src/services.rs`, `backend/src/access.rs`, `backend/migrations/` y `backend/tests/`. Extender la API administrativa; conservar el alta de wiki y aislar la llamada externa del acceso a datos.
- Frontend: `frontend/src/api.ts`, `frontend/src/views/` y `frontend/src/tests/`. Reutilizar el cliente y la vista administrativa existente; gestionar resultados asíncronos por revisión.

## Fases 0 y 1

- **Fase 0:** decisiones de persistencia, catálogo, identidad, límites y vigencia en [research.md](research.md).
- **Fase 1:** diseño local y criterios observables documentados. Riesgos a cubrir en implementación: concurrencia en el alta, permisos administrativos, contacto restringido y resultados obsoletos.
- **Bloqueo de entrega real:** no se identificó un contrato concreto de identidad SIPI en el repositorio. No inventar un adaptador ni declarar completado SC-004. El trabajo local con dobles puede avanzar; la prueba real requiere contrato, destino, operación y canal autorizados, y evidencia desde el panel.

## Complejidad

Sin excepciones constitucionales de diseño. Las tareas de implementación y verificación están definidas en [tasks.md](tasks.md). La implementación y la verificación de la entrega siguen pendientes.

## Fase final: cierre de implementación

- [ ] Actualizar `AGENTS.md` con las decisiones de diseño y convenciones nuevas de esta feature, una línea por decisión, con referencia a la spec (p. ej. «[003] ...»). No incluyas entradas por incluir, asegúrate siempre de que es información transversal y relevante para el proyecto que puedan aprovechar futuras features.
