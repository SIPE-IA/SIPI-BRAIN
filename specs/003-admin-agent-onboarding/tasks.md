---
description: "Tareas de implementación de la especificación 003"
---

# Tareas: alta de agentes y prueba de conexión

**Entradas:** [especificación](spec.md), [plan](plan.md), [investigación](research.md), [modelo](data-model.md), [contrato API/panel](contracts/agent-onboarding.md) y [guía de validación](quickstart.md).
**Estado:** las casillas marcadas tienen evidencia registrada. Las rutas de archivo son relativas a la raíz; los archivos nuevos se indican explícitamente.
**Pruebas:** Rust/Axum con SQLite temporal y Vue/Vitest. Para comportamiento nuevo, observar RED antes de implementar, luego GREEN y refactor con pruebas verdes. Si el comportamiento reutilizado ya pasa, registrar la línea base en vez de fabricar un fallo.
**Formato:** `T### [P?] [US#?]`. `[P]` identifica trabajo sobre archivos distintos que puede empezar en paralelo al cumplir su prerrequisito.

## Fase 1 — Preparación

- [X] T001 Preparar el entorno aislado de la guía, confirmar la línea base de alta de wikis y registrar los resultados de Rust y Vue, sin semillas para el caso de instalación vacía; seguir `specs/003-admin-agent-onboarding/quickstart.md`, `backend/rust-toolchain.toml` y `frontend/package.json`.

## Fase 2 — Base compartida

- [X] T002 Comprobar que `backend/tests/common/mod.rs` y `frontend/src/tests/setup.ts` permiten probar con base temporal, sesiones administradora y no administradora y respuestas controladas de la API; reutilizar los arneses y guardas de sesión/origen/CSRF existentes en `backend/src/api.rs` y `backend/src/access.rs`, sin introducir un subsistema nuevo.

## Fase 3 — US1: Registrar un agente desde el panel (P1, MVP)

**Prueba independiente:** Desde una instalación sin agentes, registrar el primero, recargar y comprobar que sigue elegible. Duplicados, datos inválidos y solicitudes no autorizadas no alteran los datos.

### Pruebas

- [X] T003 [P] [US1] Escribir y observar RED en el contrato de `GET/POST /api/admin/agents`: catálogo vacío, `201` persistido tras reapertura, elegibilidad, normalización, campos vacíos, duplicado secuencial y concurrente sin sobrescritura, denegación por sesión/rol/origen/CSRF y sin asignación nueva, en `backend/tests/agent_onboarding_registration.rs` (nuevo). FR-001, FR-004, SC-002.
- [X] T004 [P] [US1] Escribir y observar RED para **Register agent** desde catálogo vacío, duplicado, confirmación solo tras éxito, actualización de elegibilidad y aviso separado si falla el refresco posterior, en `frontend/src/tests/agent-onboarding-registration.spec.ts` (nuevo). FR-001.

### Implementación

- [X] T005 [US1] Implementar alta y listado sobre la tabla existente en `backend/src/services.rs`: «Identificador y nombre no vacíos tras quitar espacios exteriores. Alta activa. PK existente; un duplicado no cambia la fila. No incorporar otra operación de edición o baja.» Reutilizar el control administrativo vigente y no crear asignaciones.
- [X] T006 [US1] Exponer `GET/POST /api/admin/agents` en `backend/src/api.rs` conforme al contrato: `Cache-Control: no-store`, `409 agent_already_exists` y `201` solo después del commit; proyectar `connection: null` mientras no exista configuración. Llevar T003 a GREEN.
- [X] T007 [US1] Conectar **Register agent** en `frontend/src/views/WikiAdminCreateView.vue` mediante `frontend/src/api.ts`; mostrar y actualizar listado y elegibilidad, sin edición o baja. Llevar T004 a GREEN.

## Fase 4 — US2: Crear la wiki del agente (P2)

**Prueba independiente:** Con un agente activo sin wiki preparado por el arnés, crearla y recargar. Sin configuración ni asignación, el alta funciona pero la lectura continúa denegada.

### Pruebas

- [X] T008 [P] [US2] Fijar el contrato reutilizado de `POST /api/admin/wikis` en `backend/tests/wiki_admin/create.rs`: cubrir con un agente sin configuración el `201`, asociación y persistencia; comprobar rechazo para inactivo, con wiki, sin rol o CSRF, unicidad concurrente y ausencia de permiso automático. Reutilizar casos existentes antes de añadir solo los que falten. FR-002, FR-004, SC-002.
- [X] T009 [P] [US2] Probar que un agente elegible sin conexión puede seleccionarse, crear la wiki y ver su asociación al recargar; comprobar que la confirmación no implica acceso y que los errores no insertan una fila optimista, en `frontend/src/tests/wiki-admin-create.spec.ts`. FR-002, FR-004.

### Integración

- [X] T010 [US2] Reutilizar **Create wiki** y el catálogo de elegibles en `frontend/src/views/WikiAdminCreateView.vue`; comprobar la integración con el alta, o mantener el flujo sin cambios si ya satisface T009. Conservar en `backend/src/services.rs` y `backend/migrations/0002_wikis.sql` el contrato: «FK y unicidad sobre `agent_id`; solo agente activo elegible. No depende de conexión y no crea asignaciones.»
- [X] T011 [US2] Ejecutar T008 y T009 con el agente de prueba sin pasar por US1 ni US3 y comprobar la integración US1 → **Create wiki** tras T007; registrar la evidencia en `specs/003-admin-agent-onboarding/quickstart.md`.

## Fase 5 — US3: Configurar y probar la conexión (P3)

**Prueba independiente:** Con un agente de prueba ya registrado, pero sin wiki, guardar un destino aprobado y lanzar la prueba con un doble local. Distinguir identidad correcta, plazo vencido, identidad inválida y aprobación retirada. La aceptación real se valida por separado.

### Pruebas locales

- [X] T012 [P] [US3] Escribir y observar RED para el contrato de `GET /api/admin/connection-destinations`, `PUT /api/admin/agents/{agent_id}/connection` y `POST /api/admin/agents/{agent_id}/connection/test` en `backend/tests/agent_onboarding_connection.rs` (nuevo): permisos y CSRF, catálogo vacío/filtrado, actualización y persistencia de revisión, rechazo sin contacto, cuatro resultados, redirección no seguida, carrera de revisión/aprobación y ausencia de cambios de contenido o permisos y de secretos en respuestas/trazas. Usar un doble local con conteo de contactos y control del plazo. En un mismo proceso, probar con sincronización controlada: (1) activar la revocación antes del contacto, con solicitud ya en cola: `not_testable` y cero contactos; comprobar también que GET omite el destino, PUT rechaza seleccionarlo y la configuración se conserva; (2) activar la revocación durante la espera: `not_testable` aunque responda la identidad correcta; (3) revocar y volver a aprobar la misma clave durante la prueba: `not_testable` para esa prueba. FR-003 a FR-007, SC-003. Con un agente que ya tiene una wiki, capturar el estado persistido de ambos antes de ejecutar una prueba de conexión. Forzar una identidad distinta con el doble local, comprobar que el resultado es `identity_unverified` y volver a leer ambos registros. Verificar que siguen existiendo, que ninguno cambió y que la wiki conserva su asociación con el agente.
- [X] T013 [P] [US3] Escribir y observar RED en `frontend/src/tests/agent-onboarding-connection.spec.ts` (nuevo): selector solo de aprobados, catálogo vacío, configuración guardada, **Test connection** explícito, cuatro resultados, descarte de respuesta fuera de orden o de otra revisión/agente, y resultado no restaurado tras recarga. FR-003, FR-005, FR-006.

### Implementación local

- [X] T014 [US3] Crear `backend/migrations/0007_agent_connections.sql`: «`agent_id` PK/FK, `destination_id`, `expected_identity`, `revision` opaca.» Una fila por agente; no modificar filas existentes ni crear tabla de resultados.
- [X] T015 [US3] Incorporar el catálogo del entorno, vacío por defecto, en `backend/src/agent_connection.rs` (nuevo) y enlazarlo al arranque/estado en `backend/src/lib.rs` y `backend/src/main.rs`. Permitir activar atómicamente nuevas versiones en el proceso en ejecución, fuera del panel y sin reiniciar; no depender de una lectura exclusiva al arranque. Una reaprobación de la misma clave usa una nueva versión de aprobación. Respetar: «Clave y etiqueta no sensible, alcance del destino efectivo, contrato/adaptador e identidad verificable; canal autorizado si aplica.» y «Fuera de administración web. La aprobación es autoritativa; un destino retirado puede seguir referenciado en SQLite, pero no se puede volver a seleccionar ni contactar. No enviar datos internos ni autenticación al navegador.» Proveer `scripts/local-connection-fixture.cjs` (nuevo) con `serve --port --catalog` para escuchar solo en `127.0.0.1`, publicar el catálogo temporal y mostrar conteos por destino; `revoke --catalog --destination` publica una nueva versión fuera del panel. Confirmar la activación en el backend sin revelar datos sensibles.
- [X] T016 [US3] Implementar guardado y lectura de `ConnectionSummary` en `backend/src/services.rs`; integrar la proyección de T006. Aplicar: «Identidad no vacía, compatible con el contrato de identidad. Guardar solo una clave de catálogo aprobada y compatible. Generar revisión nueva en cada cambio efectivo en la misma escritura atómica; guardar los mismos valores no invalida. No guardar URL, secreto ni copia de la aprobación.» Guardar no contacta el destino.
- [X] T017 [US3] Implementar la verificación acotada en `backend/src/agent_connection.rs` y su coordinación en `backend/src/services.rs`: ordenar de forma inequívoca la activación del catálogo, el inicio del contacto y la confirmación del resultado. Revalidar la revisión de configuración y la versión de aprobación del destino antes de contactar y antes de confirmar éxito. No iniciar contacto si la aprobación vigente no es comprobable; devolver `not_testable` si la aprobación falta o dejó de ser la misma antes de confirmar, incluso si se reaprobó la clave o respondió la identidad correcta. Solo confirmar éxito con identidad comprobada y coincidente. Fuera de transacciones de escritura, límite total inicial 10 s y conexión hasta 3 s si el transporte lo permite; sin reintentos, redirecciones, descubrimiento ni intercambio. Respetar «Revisión probada y código cerrado: `verified`, `unreachable`, `identity_unverified`, `not_testable`.» y «Fuera de SQLite. No incluir respuestas remotas. Solo es aplicable a la revisión y aprobación vigentes; el panel no restaura resultados tras recargar.» Incluir el adaptador sintético solo con la opción `local-connection-fixture` en `backend/Cargo.toml` y habilitarlo solo con `SIPIBRAIN_LOCAL_FIXTURE_CATALOG` explícita en `backend/src/main.rs`. Restringirlo a loopback y sin credenciales reales; el arranque normal no registra destinos del doble.
- [X] T018 [US3] Conectar las tres rutas de T012 en `backend/src/api.rs`, con la misma sesión, rol, origen/CSRF y proyección segura del contrato. No iniciar contactos si la autorización o configuración no es válida. Llevar T012 a GREEN; no introducir un protocolo SIPI supuesto.
- [X] T019 [US3] Integrar listado, selector de aprobados, guardado y **Test connection** en `frontend/src/api.ts` y `frontend/src/views/WikiAdminCreateView.vue`; invalidar por revisión/agente/solicitud y no acoplar al alta ni a la wiki. Llevar T013 a GREEN.

### Integración y aceptación real — bloqueadas por insumos externos

- [ ] T020 [US3] Obtener del responsable contrato compatible, destino aprobado, método de verificación de identidad y autorización explícita para destino, operación y canal de autenticación. Registrar solo referencias no sensibles en `specs/003-admin-agent-onboarding/quickstart.md`; sin estos insumos, no contactar ningún servicio ni cerrar SC-004.
- [ ] T021 [US3] Con T018 y T020 cumplidas, implementar la comprobación de identidad conforme al contrato proporcionado en `backend/src/agent_connection.rs` y fijar su comportamiento con un doble del contrato aprobado en `backend/tests/agent_onboarding_real_contract.rs` (nuevo; RED antes del adaptador). No agregar lecturas de wikis, acciones de intercambio ni credenciales al código.
- [ ] T022 [US3] Con T019–T021 cumplidas, demostrar **Test connection** exitoso desde el panel con un agente SIPI real autorizado. Registrar fecha, entorno, referencia, resultado y evidencia no sensible de ausencia de intercambio o cambios de permisos en `specs/003-admin-agent-onboarding/quickstart.md`. Solo esta evidencia aprueba SC-004. FR-006, FR-007.

## Fase 6 — Validación transversal

- [X] T023 Ejecutar suites Rust y Vue, comprobación de tipos y build según `backend/Cargo.toml`, `frontend/package.json` y `specs/003-admin-agent-onboarding/quickstart.md`; revisar que guardas, API de wikis/asignaciones y aislamiento preserven el comportamiento de la 002, y registrar resultados y verificaciones pendientes en la guía.
- [X] T024 Ejecutar el recorrido local desde instalación vacía y la prueba controlada, comprobar persistencia tras reinicio y anotar evidencia SC-001 a SC-003 en `specs/003-admin-agent-onboarding/quickstart.md`; conservar SC-004 pendiente mientras T022 no tenga evidencia. Repetir desde el panel los cuatro casos del doble documentados en la guía, incluida la revocación en caliente con cero contactos nuevos. Verificar por separado cuatro configuraciones: (1) arranque normal, sin la opción `local-connection-fixture` ni `SIPIBRAIN_LOCAL_FIXTURE_CATALOG`; (2) opción de compilación sin catálogo; (3) catálogo sin la opción de compilación; y (4) ambas condiciones presentes, pero con una configuración fuera de loopback. En ninguno de los casos debe quedar habilitado el doble ni producirse contacto con un destino sintético. Para el caso fuera de loopback, usar una configuración inválida controlada, sin contactar servicios externos. El recorrido manual complementa T012 y no acredita SC-004.
- [ ] T025 Verificar el cierre constitucional en `specs/003-admin-agent-onboarding/quickstart.md`: revisar la evidencia de T024, confirmar SC-004 con T022 y vincular las pruebas aplicables al candidato para cada control. Registrar los controles no demostrados como pendientes o bloqueados. T025 no impide ejecutar T023–T024, pero debe completarse antes de cerrar la entrega.

## Dependencias y orden de ejecución

| Línea | Orden |
| --- | --- |
| Base común | T001 → T002. |
| US1 | T002 → (T003 ∥ T004) → T005 → T006 → T007. |
| US2 | T002 → (T008 ∥ T009) → T010 → T011; la prueba independiente no requiere US1, la comprobación conjunta de T011 usa T007. |
| US3 local | T002 → (T012 ∥ T013) → T014 → T015 → T016 → T017 → T018 → T019. T016 requiere T006 para actualizar la proyección existente; la prueba independiente usa un agente del arnés y no requiere US2 ni alta desde la UI. |
| US3 real | T020 puede tramitarse en paralelo sin contacto remoto; T018 + T020 → T021; T019 + T021 → T022. |
| Validación local | T007 + T011 + T019 → T023 → T024. No esperar T020–T022 para verificar SC-001 a SC-003. Si el adaptador real se incorpora después, repetir los controles afectados. |
| Cierre constitucional | T022 + T024 + evidencia constitucional aplicable → T025. Si falta evidencia, informar avance parcial; no cerrar la entrega. |

### Ejemplos de ejecución en paralelo

- **US1:** T003 (Rust) ∥ T004 (Vue), después de T002.
- **US2:** T008 (Rust) ∥ T009 (Vue), después de T002.
- **US3:** T012 (Rust) ∥ T013 (Vue), después de T002. Secuenciar los cambios que comparten API, servicios o vista entre historias y no ejecutar pruebas contra una base compartida.

## Estrategia de entrega

1. **MVP: US1.** Demostrar que el administrador registra el primer agente y lo encuentra tras recargar.
2. **Segundo incremento: US2.** Crear una wiki para ese agente sin exigir conexión ni conceder acceso.
3. **Tercer incremento: US3 local.** Configurar desde el catálogo aprobado y comprobar identidad con un doble; completar SC-001 a SC-003.
4. **Cierre de la primera entrega:** completar SC-004 con el agente SIPI real autorizado y verificar el cierre constitucional en T025. El MVP y el incremento con doble no equivalen a ese cierre.
