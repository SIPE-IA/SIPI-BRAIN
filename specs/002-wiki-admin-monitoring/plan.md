# Implementation Plan: SIPI-brain — Creación y supervisión de LLMwikis

**Branch**: `codex/002-wiki-admin-monitoring` | **Date**: 2026-11-18 | **Spec**: [Especificación 002](spec.md)

**Input**: [Especificación 002](spec.md). La 001 es un antecedente de solo lectura; no se alteran sus artefactos ni el estado de sus pendientes.

## Summary

Entregar dos flujos en el área administrativa: crear una wiki para un agente elegible y consultar su crecimiento a partir de datos persistidos. Todas las wikis muestran fechas de actividad; revelar autoría o abrir contenido requiere la asignación de lectura correspondiente. La restricción se aplica en el backend y no depende de Vue.

## Technical Context

| Área | Base y decisión |
| --- | --- |
| Lenguaje y plataforma | Rust 1.98.0; Vue 3, TypeScript y Vite; piloto web local en Windows, mismo origen. |
| Dependencias | Axum, SQLx, sesiones, Vue y Vitest del repositorio; sin nuevas dependencias. |
| Persistencia | SQLite y restricciones existentes; reutilizar el historial transaccional. No se prevé migración ni backfill para datos producidos por flujos soportados. |
| Verificación | Cargo, Vitest y comprobación de tipos/build; escenarios sintéticos aislados en [quickstart.md](quickstart.md). |
| Rendimiento | Consultar bajo demanda; acotar actividad por wiki y evitar multiplicación de conteos por cruces entre eventos, registros y fuentes. Sin SLO nuevo. |
| Alcance | Catálogo de agentes activos existente para el alta. Sin alta de agentes, eliminación, gráficas, tiempo real, puertos ni integraciones nuevas. |

## Constitution Check

| Principio | Diseño y comprobación |
| --- | --- |
| Simplicidad y alcance aprobado | Reutilizar transacciones, guardas, tablas y flujo de asignaciones sin tocar la 001. |
| Memoria basada en LLM Wiki | Contar fuentes originales y registros como entidades distintas; no modificar originales ni revisiones previas. |
| Aislamiento e intercambio mediado | Una wiki por agente; no crear asignaciones implícitas. Mantener el guard de lectura y la mediación existente sin cambios. |
| Trazabilidad verificable | Fechas y autoría desde el historial persistido de cada wiki; probar persistencia tras reinicio, aislamiento y control de acceso. |
| Contexto, no autoridad | Los metadatos no conceden permisos. Una propuesta pendiente persistida no se presenta como contenido publicado. |
| Datos mínimos y secretos | Proyección explícita; sin autoría no autorizada, identificadores de eventos, referencias a secretos o contenido. Solo datos sintéticos en verificación. |

**Resultado de diseño, antes y después de fase 1:** sin excepciones a la constitución. Las comprobaciones funcionales quedan pendientes de implementación. La prueba real de puertos/intercambio de la 001 no se sustituye ni se declara aprobada.

## Project Structure

### Documentation (this feature)

- [research.md](research.md) — decisiones y alternativas, fase 0.
- [data-model.md](data-model.md) — entidades, transiciones e invariantes.
- [contracts/wiki-admin.md](contracts/wiki-admin.md) — contrato HTTP y de interfaz.
- [quickstart.md](quickstart.md) — comprobación prevista en entorno sintético.
- `tasks.md` se generará en la fase 2 con `$speckit-tasks`.

### Source Code (repository root)

- `backend/src/{api,services,models,access,db}.rs`: agregar rutas, contratos y operaciones administrativas usando guardas y transacciones existentes. `backend/migrations/` es el antecedente del esquema; no modificar migraciones aplicadas.
- `frontend/src/{App.vue,api.ts,types.ts}` y `frontend/src/views/`: entrada administrativa, formulario y supervisión; preservar la vista de asignaciones y «My wikis».
- `backend/tests/` y `frontend/src/tests/`: pruebas públicas, datos aislados y regresiones de flujos existentes.

## Secuencia de implementación propuesta

1. **Alta (FR-001–004, SC-001–002):** observar RED de creación, permisos, persistencia y duplicados; implementar catálogo elegible, alta transaccional y confirmación en el listado. Verificar compatibilidad con asignaciones y que una creación fallida no genere una fila aparente.
2. **Supervisión (FR-005–008, SC-003–005):** observar RED de conteos, actividad, privacidad y error frente a vacío; implementar proyección de metadatos, respuesta mínima por asignación y estados de interfaz. No ampliar rutas de contenido existentes.
3. **Integración:** comprobar el escenario de dos wikis, revocación efectiva en la siguiente consulta, cierre/reapertura y rechazo fuera de la interfaz. Ejecutar suites y registrar resultados reales; la cobertura de autorización, privacidad y concurrencia requiere verificación independiente durante la implementación.

**Prerrequisito operativo:** el piloto actual crea sus agentes de ejemplo junto con sus wikis. Para ejercer el alta, verificar un agente previamente provisionado sin wiki; usar un fixture independiente en aceptación. Si no hay candidatos, el flujo muestra un estado explícito; no se incluye aprovisionamiento en la 002.
