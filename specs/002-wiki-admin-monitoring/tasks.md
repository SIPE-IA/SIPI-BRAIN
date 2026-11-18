---
description: "Tareas de implementación de la especificación 002"
---

# Tasks: SIPI-brain — Creación y supervisión de LLMwikis

**Input**: [Especificación](spec.md), [plan](plan.md), [investigación](research.md), [modelo](data-model.md), [contrato](contracts/wiki-admin.md) y [validación](quickstart.md).  
**Estado**: Implementación y verificación automatizada completadas. La 001 se conserva como antecedente de solo lectura; no se modifican sus artefactos ni se atribuyen verificaciones a sus pendientes.  
**Tests**: Test-first conforme al plan y a la constitución. Registrar RED de comportamiento con runner funcional antes de implementar y comprobar GREEN después. Verificar en la interfaz pública los estados, respuestas y efectos persistidos que correspondan.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Puede ejecutarse en paralelo con las tareas indicadas de archivos disjuntos después de sus prerrequisitos.
- **[US1]**, **[US2]**: Identifican la historia; los bloques compartidos no llevan etiqueta.
- Las rutas son relativas a la raíz. Los nombres de archivo nuevos indican destinos, no archivos ya existentes.

## Phase 1: Setup

- [X] T001 Establecer la línea base con las suites existentes: `cargo test --manifest-path backend/Cargo.toml --locked`, `npm --prefix frontend run test -- --run`, `npm --prefix frontend run typecheck` y `npm --prefix frontend run build`. No ejecutar `cargo test --manifest-path backend/Cargo.toml --locked --test wiki_admin` en esta tarea: la suite se incorporará en la 002. Registrar los resultados en `specs/002-wiki-admin-monitoring/tasks.md`. Usar un entorno local sintético aislado y la cadena existente de `backend/rust-toolchain.toml` y `frontend/package.json`; no cambiar dependencias, el entorno de la 001 ni datos del usuario.

## Phase 2: Foundational

- [X] T002 Preparar el punto de entrada `backend/tests/wiki_admin.rs` para la suite nueva, reutilizando `backend/tests/common/mod.rs` para sesión y SQLite temporal. Declarar módulos de alta y actividad sin cambiar los fixtures de otras suites. Habilitar reapertura de la misma base y solicitudes directas con y sin rol o asignación.
- [X] T003 Revisar y registrar en `specs/002-wiki-admin-monitoring/tasks.md` las guardas y contratos que deberán permanecer invariantes en `backend/src/access.rs`, `backend/src/api.rs` y `frontend/src/App.vue`: rol activo para administración, asignación para contenido e historial, origen y CSRF en mutaciones, formato de errores, acceso a «My wikis» y flujo de asignaciones. `backend/migrations/0002_wikis.sql` ya garantiza la unicidad; no modificar migraciones aplicadas.

## Phase 3: User Story 1 — Crear LLMwikis (P1, MVP)

**Objetivo**: Dar de alta una wiki para un agente elegible desde administración (FR-001–004).  
**Prueba independiente**: Usar un agente sintético activo sin wiki, crear la wiki, reiniciar, confirmar una fila persistida y asignarla mediante «Manage assignments». En solicitudes concurrentes para otro agente elegible, confirmar como máximo una creación (SC-001–002). No necesita supervisión.

### Pruebas RED

- [X] T004 [P] [US1] Escribir pruebas de `GET /api/admin/wikis` y `POST /api/admin/wikis` en `backend/tests/wiki_admin/create.rs`: fixture de agente elegible independiente del seed; selector vacío legítimo; alta y reapertura; asignación posterior sin permisos implícitos; rechazo sin sesión, rol, origen o CSRF y para nombre vacío o agente inexistente/inactivo; duplicado secuencial y concurrente con códigos, una sola fila y original intacto; fallo de guardado sin confirmación ni fila nueva. Observar RED (FR-001–004).
- [X] T005 [P] [US1] Escribir pruebas del flujo en `frontend/src/tests/wiki-admin-create.spec.ts`: entrada y formulario solo para administrador, selector sin candidatos, confirmación solo al recibir `201` válido, fila y catálogo actualizados, error sin éxito ni fila optimista y fallo de refresco separado de un alta confirmada. Conservar la navegación existente. Observar RED (FR-001, FR-003–004).

### Implementación

- [X] T006 [US1] Definir listado, entrada de alta y el error `409 wiki_already_exists` en `backend/src/models.rs`; reflejar los tipos en `frontend/src/types.ts`. Definir `AgentSummary` y `WikiAdminItem` solo si US2 aún no los incorporó. Aplicar literalmente: «Solo un agente activo y sin wiki es elegible para crear una», «`id` opaco generado por el sistema; `agent_id` único, FK a agente», «Nombre no vacío tras recortar espacios. La unicidad es por agente, no por nombre. El alta no crea contenidos ni asignaciones».
- [X] T007 [US1] Implementar listado, selector y alta en `backend/src/services.rs` y conectar ambos métodos de `/api/admin/wikis` en `backend/src/api.rs`. Reutilizar rol vigente, origen/CSRF y `db::write`. Resolver duplicados mediante la restricción existente; devolver `201` solo después del commit y `Cache-Control: no-store`. Dejar intactas las rutas de lectura y asignación.
- [X] T008 [US1] Conectar el cliente en `frontend/src/api.ts`, crear `frontend/src/views/WikiAdminCreateView.vue` e integrar «Manage wikis» y «Create wiki» en `frontend/src/App.vue`. Reflejar altas confirmadas, actualizar candidatos y distinguir fallo de refresco de fallo del alta; conservar «My wikis» y «Manage assignments».
- [X] T009 [US1] Ejecutar GREEN de `backend/tests/wiki_admin/create.rs` y `frontend/src/tests/wiki-admin-create.spec.ts`, y validar SC-001–002 de `specs/002-wiki-admin-monitoring/quickstart.md`. Registrar comandos y resultados, incluido el requisito de disponer de un agente elegible para la prueba manual.

## Phase 4: User Story 2 — Supervisar el crecimiento (P2)

**Objetivo**: Mostrar conteos y fechas de actividad en todas las wikis; la autoría y el acceso a contenido requieren asignación de lectura (FR-005–008).  
**Prueba independiente**: Preparar dos wikis con aportaciones por los flujos existentes y asignar al administrador solo una. Verificar fechas en ambas, autoría y contenido solo en la asignada, ausencia de ambos tras revocar y actualización de conteos solo en la wiki que recibió aportaciones (SC-003–005). No necesita el alta de US1.

### Pruebas RED

- [X] T010 [P] [US2] Escribir pruebas de `GET /api/admin/wiki-activity` en `backend/tests/wiki_admin/activity.rs` con dos wikis previamente provisionadas: fechas incluso sin asignación, conteos independientes (+1 registro/+2 fuentes y múltiples referencias a un original), revisión sin aumentar totales, corte a diez eventos y orden UTC con empate, exclusión de eventos que no sean aportaciones confirmadas, vacío frente a error, denegación a no administradores y vigencia tras revocar. Verificar en respuesta directa ausencia de identidades, referencias, cuerpos y secretos, y mantener denegación de contenido sin asignación. Observar RED (FR-005–008).
- [X] T011 [P] [US2] Escribir pruebas en `frontend/src/tests/wiki-admin-activity.spec.ts`: carga, vacío confirmado, error de carga o refresco sin fingir ceros, conteos separados, actualización manual, fechas aunque no haya autor y autoría solo cuando el servidor la devuelve. Tras revocación, la siguiente respuesta no deja autoría obsoleta en pantalla. Observar RED (FR-005–008).

### Implementación

- [X] T012 [US2] Definir la proyección de actividad en `backend/src/models.rs` y `frontend/src/types.ts` conforme a `specs/002-wiki-admin-monitoring/contracts/wiki-admin.md`: última fecha UTC RFC 3339 o `null`, fecha por entrada obligatoria, `author_name` omitido sin lectura y sin IDs de autor. Aplicar «`sequence` solo desempata internamente; no se expone». No reutilizar el DTO de historial completo. Si US1 aún no existe, definir los tipos compartidos de wiki/agente al iniciar esta historia.
- [X] T013 [US2] Implementar la lectura coherente de rol, asignación, conteos y actividad en `backend/src/services.rs` y conectar `GET /api/admin/wiki-activity` en `backend/src/api.rs` con `Cache-Control: no-store`. Incluir todas las wikis; aplicar «Contar cada fila una vez en su wiki; editar no aumenta este total» y «Contar cada original persistido una vez, con o sin referencias; no deduplicar por hash». Usar la lista positiva de `specs/002-wiki-admin-monitoring/research.md`, limitar a diez fechas recientes por wiki y calcular la última fecha sobre todos los eventos elegibles. Usar la semántica de `backend/src/access.rs` sin relajar ninguna guarda; no proyectar autoría sin lectura ni devolver ceros ante error.
- [X] T014 [US2] Conectar la consulta en `frontend/src/api.ts`, crear `frontend/src/views/WikiAdminActivityView.vue` e integrarla para el administrador en `frontend/src/App.vue`. Conservar fechas si falta autoría, separar carga/error/vacío, reemplazar datos al actualizar y no usar esta vista como permiso para abrir contenido. Si US1 aún no existe, habilitar la entrada administrativa para esta vista sin exigir el formulario de alta.
- [X] T015 [US2] Ejecutar GREEN de `backend/tests/wiki_admin/activity.rs` y `frontend/src/tests/wiki-admin-activity.spec.ts`, y validar SC-003–005 de `specs/002-wiki-admin-monitoring/quickstart.md` usando wikis preexistentes. Registrar evidencia tanto de la interfaz como de las respuestas directas.

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T016 Ejecutar suites completas, typecheck y build con los comandos de `specs/002-wiki-admin-monitoring/quickstart.md`. Comprobar que navegar, leer y asignar wikis sigue funcionando según los contratos existentes; hacer la comparación contra la línea base de las interfaces compartidas que se hayan modificado y registrar diferencias y comprobaciones pendientes en `specs/002-wiki-admin-monitoring/tasks.md`.
- [X] T017 Obtener verificación independiente tras las comprobaciones propias para las fronteras de autorización y concurrencia de `backend/src/api.rs` y `backend/src/services.rs`. Usar base sintética aislada; contrastar ausencia de filtraciones sin asignación, revocación y unicidad concurrente con todo el contrato `specs/002-wiki-admin-monitoring/contracts/wiki-admin.md`. Registrar el resultado verificable, no presuponer aprobación.
- [X] T018 Conciliar FR-001–008 y SC-001–005 con evidencia real en `specs/002-wiki-admin-monitoring/tasks.md` y actualizar el estado de la guía en `specs/002-wiki-admin-monitoring/quickstart.md`. Verificar que los nueve archivos de `specs/001-sipibrain-collective-memory/` no cambiaron. Declarar el prerrequisito del agente elegible y las comprobaciones no ejecutadas.

## Dependencies & Execution Order

- **Base**: T001 → T002 → T003. La base debe estar lista para ambas historias.
- **US1**: T004 y T005 → T006 → T007 → T008 → T009.
- **US2**: T010 y T011 → T012 → T013 → T014 → T015.
- **Cierre**: T009 y T015 → T016 → T017 → T018.
- **Entre historias**: No existe dependencia funcional. Los cambios que comparten `models.rs`, `services.rs`, `api.rs`, `types.ts`, `api.ts` o `App.vue` se integran en serie para evitar colisiones; la primera historia que se implemente define los tipos comunes y la segunda los reutiliza.

## Parallel Examples

- **US1**: Después de T003, preparar T004 (contratos y persistencia Rust) en paralelo con T005 (comportamiento Vue).
- **US2**: Después de T003, preparar T010 (proyección y privacidad Rust) en paralelo con T011 (estados y presentación Vue). Las pruebas de US2 usan el fixture existente, no el alta de US1.

## Implementation Strategy

1. **MVP**: completar base y US1, verificar SC-001–002 y demostrar el alta sin depender de la supervisión.
2. **Siguiente incremento**: completar US2 con dos wikis ya disponibles y comprobar SC-003–005, incluidas las denegaciones fuera de la interfaz.
3. **Cierre**: ejecutar verificación conjunta y registrar resultados observados. No ampliar el alcance a aprovisionamiento de agentes, puertos, intercambio real, eliminación, gráficas o tiempo real.

## Evidencia de implementación

- T001: Cargo 28 pruebas; Vitest 14; typecheck y build verdes antes de los cambios. Node 24.22.1 ya instalado, usado mediante PATH del proceso; no se cambió la configuración global.
- T002–T003: fixture temporal aislado; se mantienen origen/CSRF y no-store del middleware, el rol activo en servidor y la asignación para rutas existentes. No se cambiaron migraciones.
- T004–T009: RED de dos pruebas Rust por 404 en lugar de contrato y una Vue por falta de entrada de alta. GREEN en Rust (2) y Vue (1); prueba de reapertura conserva la wiki, no da asignación implícita y las altas concurrentes dejan una única fila. Vue distingue un fallo de guardado de un fallo de refresco posterior a la confirmación.
- T010–T015: RED en Rust (ruta aún ausente) y Vue (sin entrada de actividad). GREEN en Cargo suite focal (4 en conjunto) y Vue actividad (1). La verificación directa muestra fechas en ambas wikis, autoría y acceso a contenido solo con lectura y ausencia de ambos al revocar. Se comprobó orden, diez eventos, conteos separados y error distinto de vacío. La comprobación de interfaz usa Vue montado con respuestas simuladas, complementada por solicitudes directas a la aplicación Rust.
- El servicio local existente usa el ejecutable habitual. Para las pruebas de la 002 se emplea CARGO_TARGET_DIR=backend/target/spec002 sin detener ni sustituir ese proceso.
- T016: suites completas: Cargo 32/32 (28 previas + 4 nuevas), Vitest 16/16 (14 previas + 2 nuevas), typecheck y build aprobados. Las regresiones existentes de acceso, asignación y cliente HTTP conservan los resultados de la línea base. Typecheck repetido por el coordinador: aprobado.
- T017: verificación independiente en entorno temporal (4 pruebas de la 002 y 3 regresiones de autorización, 7/7). Una única alta concurrente, fechas en ambas wikis, autoría omitida al revocar y lectura directa denegada sin asignación; sin bloqueos.
- T018: FR-001–004 / SC-001–002 comprobados por las pruebas de alta; FR-005–008 / SC-003–005 por supervisión y regresiones de acceso. Los nueve hashes de la 001 y HEAD coinciden con la línea base. No se ejecutó una demostración manual en navegador. Para la demostración del alta se requiere un agente activo sin wiki, no presente en el seed. Revisión nativa no iniciada: la evaluación no pudo aislar el cambio de archivos preexistentes del árbol de trabajo.
