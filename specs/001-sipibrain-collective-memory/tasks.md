---
description: "Tareas de implementación del piloto SIPI-brain con Rust y Vue 3 + TypeScript"
---

# Tasks: SIPI-brain — LLMwikis por agente

**Input**: documentos de `specs/001-sipibrain-collective-memory/`.
**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contrato de aplicación](contracts/application.md), [contrato de agentes](contracts/agent-exchange.md), [guía](quickstart.md), [AGENTS.md](../../AGENTS.md) y [constitución](../../.specify/memory/constitution.md).
**Estado**: 67 tareas propuestas; ninguna implementada ni verificada por esta generación. No se crean aplicaciones, manifests o servicios al generar este documento.
**Tests**: incluidos por los escenarios/pruebas observables de la especificación, la sección Verificación de la constitución, el plan y la disciplina test-first vigente. Observar RED de comportamiento con runner funcional antes de implementar; un fallo por runner ausente no acredita RED.
**Organization**: cinco historias P1 según la especificación. US1 corresponde a HU2; US2 a HU1; US3 a HU3; US4 a HU4 vigente; US5 a HU5.

## Format: `[ID] [P?] [Story] Description`

Todos los ítems ejecutables usan `- [ ] Tnnn [P?] [USn?]`, ruta concreta y dependencias. `[P]` permite solo los lotes disjuntos documentados después de completar sus prerrequisitos; no obliga a delegar, no autoriza efectos externos y no significa «puede empezar sin sus dependencias».

## Path Conventions

Rutas de código relativas a la raíz: `backend/` para Rust y `frontend/` para Vue 3 + TypeScript. Rutas documentales completas desde raíz o enlaces relativos a este directorio. No stack alternativo, SIPI NEXUS, restauración general de versiones, eliminación autónoma, servicios remotos ni valores secretos reales.

Los campos, relaciones y restricciones de data-model.md son parte de cada tarea de entidad, no decisiones discrecionales del implementador. En las tareas se citan las restricciones aplicables; no se inventan longitudes o nulabilidad distintas. Identificadores estables, tiempos UTC y autoría/roles derivados del servidor; `wiki_id` obligatorio para conocimiento.

**Fronteras de seguridad**: ninguna asignación de wiki o rol administrativo crea una operación de revelado de secretos. Los agentes usan principal verificado y servicios compartidos; un requester/author recibido no autentica. Las fuentes originales no se cambian al sintetizar; revisión de conocimiento es humana y trazable, no ejecución de texto recuperado.

## Phase 1: Setup — estructura y contratos compartidos

**Goal**: Inicializar solo la estructura definida en plan.md, sin crear funcionalidades ajenas ni afirmar compatibilidad antes de compilar.

**Independent Test**: Los manifiestos y comandos locales funcionan después de implementarlos; esta generación no los ejecuta.

- [X] T001 [P] Crear `backend/Cargo.toml`, `backend/src/main.rs` y `backend/src/lib.rs`; resolver la línea compatible Axum/Tokio/serde/SQLx/sesiones/Argon2 y dependencia Windows específica del target descrita en research.md, fijar toolchain según MSRV real y generar `backend/Cargo.lock` sin instalar servicios externos.
- [X] T002 [P] Crear `frontend/package.json`, `frontend/package-lock.json`, `frontend/tsconfig.json`, `frontend/vite.config.ts` y `frontend/src/main.ts` para Vue 3 + TypeScript/Vite; scripts dev/build/typecheck/test con vue-tsc/Vitest y proxy `/api` al backend loopback, sin CORS general.
- [X] T003 Fijar DTO y códigos del contrato en `backend/src/models.rs`, `frontend/src/types.ts` y `specs/001-sipibrain-collective-memory/contracts/application.md`: incluir la interfaz local mínima de WikiMaintenance/KnowledgeReview exigida por la constitución, actualmente no enumerada como rutas; documentar validación/asignación y prueba de persistencia antes de exponerla, sin inventar endpoints de agentes. Completar además el contrato de GET registros con `project_id` opcional para consultar prompts, decisiones, notas y RFDs de un proyecto dentro de la wiki autorizada; combinarlo con texto/agente/cliente y conservar el comportamiento sin filtro de proyecto. Preparar harness aislado y dobles en `backend/tests/common/mod.rs` y `frontend/src/tests/setup.ts`. (Requiere T001, T002.)

**Checkpoint**: completar y observar los prerrequisitos antes de depender de ellos; no marcar tareas por la mera existencia de archivos.

---

## Phase 2: Foundational — seguridad y persistencia base

**Goal**: Proveer las fronteras compartidas obligatorias antes de las historias.

**Independent Test**: Login/logout/CSRF, permisos por wiki, bootstrap y asignaciones se comprueban directamente contra la API, no solo con botones.

- [X] T004 [P] Escribir primero pruebas RED de credenciales, sesión persistente/rotada, logout, CSRF/Origin y denegaciones de administrador/asignación en `backend/tests/security.rs`; comparar estado de negocio antes/después y no confundir nuevos eventos sanitizados con cambios de permisos. Añadir RED de arranque en `backend/tests/server_startup.rs`: invocar serve con bind loopback y static-dir temporal de prueba, comprobar listener HTTP, GET `/` y un asset, error JSON no HTML para `/api` desconocida y fallo no exitoso ante directorio estático inválido; fixtures fuera del workspace, sin contactar agentes. (Requiere T003.)
- [X] T005 [P] Escribir primero pruebas RED del estado prelogin, token CSRF vigente, login/logout y errores sin reflejar contraseña en `frontend/src/tests/auth.spec.ts`, usando DTO de T003 y dobles explícitos de la API. (Requiere T003.)
- [X] T006 Implementar identidad/sesiones en `backend/migrations/0001_identity.sql`, `backend/src/auth.rs` y `backend/src/models.rs`: «nombre de usuario único, hash PHC Argon2id, estado activo y rol»; «Backend Rust autentica y comprueba el rol; sin contraseñas predeterminadas ni rol recibido del navegador.»; «Sesión persistente de tower-sessions/SQLx; rotar al login e invalidar al logout. Sin valores secretos, contraseñas o roles de confianza copiados del cliente.» (Requiere T004, T005.)
- [X] T007 Implementar Agent/Wiki/WikiAssignment en `backend/migrations/0002_wikis.sql`, `backend/src/models.rs` y `backend/src/access.rs`: «No se confía en un nombre recibido para autenticar un agente.»; «Unicidad de `agent_id`: una wiki propia por agente.»; «Par usuario/wiki único; solo administrador gestiona asignaciones.»; «Lectura, búsquedas y aportaciones siempre requieren esta relación. Administrar asignaciones no da acceso al contenido por una ruta alternativa.» (Requiere T006.)
- [X] T008 Conectar sesión/CSRF/Origin y `/api/session`, `/api/login`, `/api/logout` en `backend/src/api.rs`; implementar configuración loopback, errores sanitizados y SQLite/SQLx en `backend/src/config.rs` y `backend/src/db.rs`, y subcomandos interactivos init-db/bootstrap-admin/create-user en `backend/src/main.rs`, sin contraseñas en argumentos ni captura de cuerpos sensibles. Implementar también el subcomando serve con `--bind` y `--static-dir`, iniciar listener Tokio/Axum y router, y servir el bundle Vue compilado y sus assets mediante tower-http desde el directorio configurado; resolver la ruta conforme al contrato, no exponer archivos fuera del bundle, aislar `/api` del fallback HTML y dar GREEN a server_startup.rs. (Requiere T007.)
- [X] T009 Implementar GET/POST `/api/admin/assignments` en `backend/src/api.rs` y `backend/src/services.rs`, reutilizando access.rs: solo administrador, entidades existentes, sin autoasignación de no administradores ni acceso lateral a contenido/secretos; dar GREEN a las pruebas correspondientes. (Requiere T008.)
- [X] T010 Implementar cliente de sesión/CSRF y errores en `frontend/src/api.ts`, `frontend/src/App.vue` y `frontend/src/views/LoginView.vue`; usar cookies de servidor, no almacenar sesión/contraseña en localStorage y dar GREEN a auth.spec.ts. (Requiere T008.)
- [X] T011 Implementar `frontend/src/views/AssignmentsView.vue` y su integración en `frontend/src/App.vue` para gestionar asignaciones por administrador; la vista no sustituye comprobaciones del backend y no permite consultar contenido desde la administración. (Requiere T009, T010.)
- [X] T012 Ejecutar cargo check/test del backend y typecheck/Vitest/build del frontend sobre `backend/Cargo.toml` y `frontend/package.json`; ejecutar el binario con serve, bind loopback y `--static-dir frontend/dist` y comprobar `/`, un asset real de Vue y error JSON no HTML de API desconocida. Registrar comandos, salida y RED→GREEN real en `specs/001-sipibrain-collective-memory/tasks.md` antes del checkpoint, sin marcar éxito si no existe runner o falla una prueba requerida. (Requiere T011, T009.)

**Checkpoint**: completar y observar los prerrequisitos antes de depender de ellos; no marcar tareas por la mera existencia de archivos.

---

## Phase 3: User Story 1 — aportación colectiva por wiki (P1) — MVP

**Goal**: HU2: dos programadores aportan a la LLMwiki propia de un agente, con fuentes, revisión y conflictos preservados.

**Independent Test**: Dos programadores asignados guardan prompts, reabren y recuperan ambas autorías; una wiki ajena se deniega; dos ediciones concurrentes quedan pendientes hasta resolución del administrador.

Las pruebas indicadas van antes del comportamiento. Tras GREEN, refactorizar sin perder cobertura. Las tareas que implementan modelos conservan sus constraints del diseño.

- [X] T013 [P] [US1] Escribir pruebas RED de reinicio, fuentes/revisiones inmutables, pertenencia wiki/proyecto/cliente, referencias/enlaces y estado de revisión en `backend/tests/persistence.rs`; probar clientes homónimos y hash de original sin atribuirle veracidad. (Requiere T012.)
- [X] T014 [P] [US1] Escribir pruebas RED en `backend/tests/conflicts.rs` con SQLite en archivo y conexiones independientes: dos bases iguales, ambas propuestas tras reinicio, idempotencia, ID reutilizado con otro contenido, resolución de no administrador/obsoleta, nuevas candidatas y espera agotada sin falso guardado. (Requiere T012.)
- [X] T015 [P] [US1] Escribir pruebas RED de contrato fuente/registro/propuesta/resolución en `backend/tests/wiki_contract.rs`: 201, 202 state=pending, 409 idempotency_conflict/conflict_changed, 503 storage_busy y respuestas genéricas de inaccesibilidad sin mutación; texto con instrucciones no concede permisos. (Requiere T012.)
- [X] T016 [P] [US1] Escribir pruebas RED de aportación, revisión/procedencia, conflicto pendiente y resolución administrativa en `frontend/src/tests/wiki.spec.ts`; conservar texto ordinario si no se confirma guardado, sin tratar una propuesta preservada como pérdida. (Requiere T012.)
- [ ] T017 [US1] Crear Client/Project/Record en `backend/migrations/0003_knowledge.sql` y `backend/src/models.rs`: «El nombre no es identificador único; admite clientes homónimos sin mezclarlos.»; «`client_id` opcional» y «Proyecto identificable incluso sin cliente externo. Cliente no se deduce por similitud de nombres.»; Record: «`project_id` opcional, `client_id` opcional» y «`kind`: prompt, decision, note o rfd. Un RFD es un requerimiento funcional y exige proyecto. Si existen proyecto y cliente explícito, deben coincidir. `state`: active o conflicted.» (Requiere T013, T014, T015, T016.)
- [ ] T018 [US1] Implementar OriginalSource/Revision/SourceReference/RecordLink en `backend/migrations/0004_sources.sql`, `backend/src/models.rs` y servicios de `backend/src/services.rs`: «`project_id` opcional», «`parent_revision_id` opcional», «localizador opcional»; «Original inmutable, seleccionado para ingestión, no captura indiscriminada de conversaciones. No se actualiza mediante síntesis. El hash comprueba integridad, no veracidad.»; «Contenido inmutable. Distinguir hecho verificado, decisión y suposición; no convertir una categoría en otra por recuperarla. No ofrece una operación de restauración.»; «Solo fuentes de la misma wiki.» e «Interrelación explícita dentro de la misma wiki y sujeta a asignaciones; no concede acceso transversal.» (Requiere T017.)
- [ ] T019 [US1] Implementar WikiMaintenance/KnowledgeReview/WikiEvent e índice derivado en `backend/migrations/0005_review_events.sql` y `backend/src/services.rs`: «Revisión humana de referencias, contradicciones y obsolescencia. No ejecuta instrucciones recuperadas ni crea un agente de limpieza.»; «La consulta muestra revisión y procedencia; algo sin verificar no se presenta como hecho confirmado. Si cambia la revisión, no hereda silenciosamente la revisión humana anterior.»; «Registro cronológico de metadatos; sin valores secretos, cuerpos de solicitudes, contraseñas o tokens. Los eventos de una acción rechazada no revelan contenido ajeno.» (Requiere T018.)
- [ ] T020 [US1] Crear EditProposal/EditConflict/ConflictResolution en `backend/migrations/0006_conflicts.sql` y `backend/src/models.rs`: «UUID de envío único por autor/operación; mismas claves con contenido diferente se rechazan. Base y propuesta deben pertenecer al mismo registro y wiki. El contenido se conserva aunque se detecte conflicto.»; «A lo sumo un conflicto abierto por registro; dos o más propuestas conservadas. La generación cambia si llega otra aportación. Estado pending o resolved.»; «Resolución manual autorizada y atómica; conserva las propuestas y la decisión. No modifica el texto de una fuente original.» (Requiere T019.)
- [ ] T021 [US1] Implementar guardado normal e ingestión/propuestas en `backend/src/services.rs` y `backend/src/db.rs`: derivar autor del principal autenticado, comprobar wiki/asignación dentro de transacción corta BEGIN IMMEDIATE, conservar ambas ramas ante base obsoleta, no elegir ganador en conflicto y aplicar idempotencia/reintentos acotados sin red o bóveda dentro de la transacción. (Requiere T020.)
- [ ] T022 [US1] Implementar resolución manual en `backend/src/services.rs`: administrador con asignación, generación esperada, resultado y evento atómicos, propuestas intactas; nuevas candidatas invalidan resolución obsoleta y una aportación tardía no borra decisiones previas; dar GREEN a conflicts.rs. (Requiere T021.)
- [ ] T023 [US1] Implementar rutas de wikis asignadas, fuentes, registros, detalle/índice básico, propuestas y conflictos en `backend/src/api.rs`, reutilizando access.rs/services.rs; prohibir mutación de originales y permisos derivados de campos JSON; dar GREEN a wiki_contract.rs sin crear endpoints externos de agentes. (Requiere T022.)
- [ ] T024 [P] [US1] Implementar aportaciones y detalle básico en `frontend/src/views/WikiView.vue` y `frontend/src/components/RecordEditor.vue` usando DTO/API ya fijados: texto escapado, autoría/proyecto, confirmación tras commit y texto ordinario retenido ante fallo. (Requiere T023.)
- [ ] T025 [P] [US1] Implementar lista/candidatas/resolución en `frontend/src/views/ConflictsView.vue`: dos propuestas pendientes distinguibles, sin ganador implícito, administrador resuelve y 409 exige recargar/revisar antes de otra decisión. (Requiere T023.)
- [ ] T026 [US1] Integrar revisión humana, fuentes/referencias/enlaces, reglas no ejecutables e índice/eventos en `frontend/src/views/KnowledgeReviewView.vue`, `frontend/src/App.vue` y `backend/src/api.rs`, conforme a la interfaz local cerrada en T003; usar asignaciones y servicios de T019, sin análisis semántico ni limpieza automática. (Requiere T024, T025, T019.)
- [ ] T027 [US1] Implementar seed-pilot --synthetic en `backend/src/main.rs` y fixtures de `backend/tests/common/mod.rs`: dos agentes/wikis, proyectos Alfa/Beta, RFD-001/002/003 de Alfa, RFD-004 de Beta y nota propia de Estimador; nombres no son claves, repetir no duplica ni sobrescribe y no crea contraseñas utilizables conocidas. (Requiere T026.)
- [ ] T028 [US1] Validar US1/SC-001/SC-006/SC-007 en `backend/tests/persistence.rs`, `backend/tests/conflicts.rs` y `frontend/src/tests/wiki.spec.ts`, más flujo navegador/API tras reinicio; registrar resultados en `specs/001-sipibrain-collective-memory/tasks.md`, sin presentar Vitest como e2e ni aceptar fuentes modificadas. (Requiere T027.)

**Checkpoint**: cerrar solo con evidencia del criterio independiente. Mantener pendientes los pasos externos; conformidad sintética y aceptación real son resultados distintos.

---

## Phase 4: User Story 2 — RFDs entre agentes (P1)

**Goal**: HU1: Estimador recupera requerimientos funcionales de Insight con mediación autorizada y ámbito correcto.

**Independent Test**: Con fixtures, permiso/denegación y puertos se distinguen; prueba real demuestra RFDs Alfa, no Beta. Los pasos externos no se aceptan con mocks.

Las pruebas indicadas van antes del comportamiento. Tras GREEN, refactorizar sin perder cobertura. Las tareas que implementan modelos conservan sus constraints del diseño.

- [ ] T029 [P] [US2] Escribir pruebas RED del contrato interno en `backend/tests/exchange_contract.rs`: identidad falsa, destino no configurado, mediación ausente/denegada/caducada/de otro ámbito, timeout y resultado posterior a transferencia desconocido; nunca entregar contenido ajeno por un puerto accesible. (Requiere T028.)
- [ ] T030 [P] [US2] Escribir pruebas RED en `backend/tests/project_scope.rs` para cuatro RFDs individuales, proyecto obligatorio, cliente cuando corresponda, cliente/proyecto incoherente y respuesta con RFD sobrante de Beta que no debe reenviarse; usar fixtures autorizados, no servicios reales. (Requiere T028.)
- [ ] T031 [US2] Crear ExchangeReceipt/AgentEndpoint en `backend/migrations/0007_exchange.sql` y `backend/src/models.rs`: «Decisión verificable vinculada exactamente a solicitud e identidades. No acepta una bandera `approved` enviada por el solicitante como prueba. Registra referencias, no contenido sensible ni credenciales.»; incluir fecha/vencimiento; «Solo configuración explícita de un responsable. Nada de descubrimiento automático, credenciales ambientales o rutas supuestas. Una referencia no contiene un valor secreto.» (Requiere T029, T030.)
- [ ] T032 [US2] Implementar fronteras Rust check_connection/authorize_exchange/retrieve_authorized y dobles explícitos en `backend/src/integrations.rs` y `backend/src/config.rs`; separar disponibilidad de transporte, identidad y recuperación; destinos reales deshabilitados por defecto, plazos acotados del contrato y sin reintento ciego de efectos desconocidos. (Requiere T031.)
- [ ] T033 [US2] Implementar recuperación RFD por proyecto/cliente en `backend/src/services.rs`, reutilizando los registros de US1 y access.rs; mantener requerimientos individuales, procedencia y ausencia explícita, sin imponer formato documental ni deducir cliente por nombre. (Requiere T032.)
- [ ] T034 [US2] Conectar decisión verificada, recibo y transferencia permitida en `backend/src/services.rs` y `backend/src/integrations.rs`; vincular solicitante/origen/destinatario/propósito/referencias/proyecto/vencimiento, registrar copia autorizada como fuente local con recibo si se persiste, no referencia que eluda la wiki original ni red dentro de transacción. (Requiere T033.)
- [ ] T035 [US2] Dar GREEN y ejecutar `backend/tests/exchange_contract.rs` y `backend/tests/project_scope.rs`; registrar por separado conformidad del doble y SC-002 bloqueado/no ejecutado en `specs/001-sipibrain-collective-memory/tasks.md`, sin frenar US3 o US5 por falta de servicios reales. (Requiere T034.)
- [ ] T036 [US2] BLOQUEO EXTERNO: obtener de responsables contratos/destinos/puertos, identidad, sesión autorizada y mediador disponible; registrar referencias sanitizadas, responsable, alcance y estado en `specs/001-sipibrain-collective-memory/contracts/live-services.md`. No descubrir datos por escaneo ni capturar credenciales; sin entrada, mantener las tareas reales siguientes pendientes. (Requiere T035.)
- [ ] T037 [US2] BLOQUEO EXTERNO: solo con T036 satisfecho, implementar los adaptadores reales de los contratos suministrados en `backend/src/integrations.rs` y su configuración explícita; no traducir nombres internos a endpoints supuestos ni incorporar SIPI NEXUS. (Requiere T036.)
- [ ] T038 [US2] BLOQUEO EXTERNO: implementar/ejecutar la prueba explícita de puertos e intercambio permitido/denegado en `backend/tests/live_agents.rs`; exigir casos realmente ejecutados y ausencia de contenido Beta, fallar/bloquear si falta autorización, registrar evidencia sanitizada en `specs/001-sipibrain-collective-memory/tasks.md`. SC-002 completo exige además la conversación nueva de US4. (Requiere T037.)

**Checkpoint**: cerrar solo con evidencia del criterio independiente. Mantener pendientes los pasos externos; conformidad sintética y aceptación real son resultados distintos.

---

## Phase 5: User Story 3 — consulta manual (P1)

**Goal**: HU3: buscar y abrir conocimiento autorizado desde Vue sin depender de agentes reales.

**Independent Test**: Con fixtures autorizados, las tres búsquedas originales abren el mismo RFD; además, filtrar por project_id recupera prompts, decisiones, notas y RFDs solo del proyecto elegido aunque otro proyecto comparta cliente y texto. Combinar o limpiar filtros conserva resultados correctos; ausencia y wiki ajena no revelan información.

Las pruebas indicadas van antes del comportamiento. Tras GREEN, refactorizar sin perder cobertura. Las tareas que implementan modelos conservan sus constraints del diseño.

- [ ] T039 [P] [US3] Escribir pruebas RED de búsqueda/detalle/ausencia e inaccesibilidad en `backend/tests/search.rs`, con asignaciones manipuladas, homónimos y consultas parametrizadas; conservar el mismo RFD y procedencia por los tres filtros originales. Añadir FR-004: dos proyectos del mismo cliente y wiki con texto coincidente y registros de tipos prompt, decision, note y rfd; filtrar por `project_id` devuelve solo los cuatro tipos del proyecto seleccionado, combinado con texto/agente/cliente, sin restringirse a RFDs. Sin project_id conservar resultados previos; proyecto ajeno/inexistente produce lista vacía sin revelar su existencia y no altera estado de negocio. (Requiere T028.)
- [ ] T040 [P] [US3] Escribir pruebas RED de búsquedas, apertura del mismo RFD, estado vacío y errores genéricos en `frontend/src/tests/search.spec.ts`, usando DTO estables y fixtures sin servicios reales. Añadir selección de `project_id`, envío del parámetro y combinación con filtros existentes: prompts, decisiones, notas y RFDs del proyecto elegido, exclusión del otro proyecto del mismo cliente, estado vacío y limpieza del filtro sin romper las tres búsquedas originales. (Requiere T028.)
- [ ] T041 [US3] Implementar consultas SQLx filtradas por asignación antes de buscar/contar y sus filtros de contenido, agente, cliente y `project_id` opcional en `backend/src/services.rs` y `backend/src/db.rs`; aplicar proyecto a todos los tipos prompt, decision, note y rfd, combinar los filtros y no deducir proyecto por cliente o nombre. Solo conocimiento de la wiki, sin reservas de secretos ni contenido ajeno; ausencia del parámetro conserva el comportamiento anterior y un proyecto sin registros accesibles devuelve lista vacía sin filtrar metadatos ajenos. (Requiere T039, T040.)
- [ ] T042 [US3] Extender GET registros/detalle en `backend/src/api.rs` mediante el servicio existente y el contrato actualizado en T003: validar y propagar `project_id` opcional, combinable con contenido/agente/cliente para todos los tipos de registro. Mantener resultados vacíos explícitos y 404 genérico para wiki/registro inaccesibles; no generar antecedentes ante ausencia, devolver información de otro proyecto ni crear una ruta de administración alternativa. (Requiere T041.)
- [ ] T043 [P] [US3] Implementar `frontend/src/views/SearchView.vue` con consultas, paginación, campo de filtro `project_id` y estados vacío/error; permitir aplicar, combinar y limpiar el proyecto sobre prompts, decisiones, notas y RFDs, reiniciando paginación al cambiar filtro. No inferir permisos en cliente ni mostrar conteos de wikis ajenas. (Requiere T042.)
- [ ] T044 [P] [US3] Implementar `frontend/src/views/RecordView.vue` para contenido escapado, procedencia, proyecto/cliente, fuentes y estado de revisión/conflicto; mostrar propuestas pendientes sin atribuir vigencia automática. (Requiere T042.)
- [ ] T045 [US3] Integrar las vistas y peticiones en `frontend/src/App.vue`, `frontend/src/api.ts` y `frontend/src/types.ts`, reutilizando clientes existentes y el contrato de T003; tipar y enviar `project_id` solo cuando se aplique, mantener los demás filtros y no cambiar controles de sesión/CSRF. (Requiere T043, T044.)
- [ ] T046 [US3] Ejecutar backend `backend/tests/search.rs` y frontend `frontend/src/tests/search.spec.ts` y validar con navegador/API locales el mismo RFD por los tres filtros originales, más FR-004 por `project_id` para prompts, decisiones, notas y RFDs entre dos proyectos del mismo cliente. Comprobar combinación y limpieza del filtro, resultados vacíos y aislamiento; registrar SC-003/SC-004, cobertura FR-004 y rechazos sin mutación en `specs/001-sipibrain-collective-memory/tasks.md`. (Requiere T045.)

**Checkpoint**: cerrar solo con evidencia del criterio independiente. Mantener pendientes los pasos externos; conformidad sintética y aceptación real son resultados distintos.

---

## Phase 6: User Story 4 — continuidad entre conversaciones (P1)

**Goal**: HU4 vigente: recuperar memoria persistida desde otra conversación sin reenviar el historial.

**Independent Test**: Tras guardar y reiniciar, un principal de agente verificado inicia otra conversación y recupera antecedentes; el caso extranjero sigue sujeto a mediación.

Las pruebas indicadas van antes del comportamiento. Tras GREEN, refactorizar sin perder cobertura. Las tareas que implementan modelos conservan sus constraints del diseño.

- [ ] T047 [P] [US4] Escribir pruebas RED de nueva conversación sin historial y reinicio con SQLite en archivo en `backend/tests/continuity.rs`; propia wiki y antecedentes permitidos conservan referencias/revisión y no piden reintroducir información. (Requiere T028, T035.)
- [ ] T048 [P] [US4] Escribir pruebas RED en `backend/tests/agent_identity.rs`: requester/author/agent_id/rol del JSON no autentican, nueva conversación no concede permisos, contexto con instrucciones no amplía acceso y mediación ausente no habilita otra wiki. (Requiere T028, T035.)
- [ ] T049 [US4] Implementar recuperación y aportación de la propia wiki mediante principal de agente ya verificado en `backend/src/services.rs` y `backend/src/integrations.rs`, reutilizando guardado/autorización de US1; no añadir endpoints externos supuestos, memoria volátil paralela o restauración de versiones. (Requiere T047, T048.)
- [ ] T050 [US4] Integrar contexto recuperado y recibos autorizados de US2 en `backend/src/integrations.rs`, conservando identidad/proyecto/procedencia/categoría y estado pendiente; tratar los datos como contexto, nunca como instrucciones o autorización. (Requiere T049.)
- [ ] T051 [US4] Ejecutar continuity.rs/agent_identity.rs y verificar nueva conversación con dobles y reinicio en `backend/tests/continuity.rs`; registrar resultado sintético separado del caso real en `specs/001-sipibrain-collective-memory/tasks.md`. (Requiere T050.)
- [ ] T052 [US4] BLOQUEO EXTERNO: con T038 aprobado y núcleo US4 listo, completar SC-002 en `backend/tests/live_agents.rs`: nueva conversación de Estimador, tres RFDs Alfa desde Insight y nota propia, no Beta, sin reintroducir antecedentes; evidenciar mediación y denegación, no aprobar con mocks ni cero pruebas. (Requiere T051, T038.)

**Checkpoint**: cerrar solo con evidencia del criterio independiente. Mantener pendientes los pasos externos; conformidad sintética y aceptación real son resultados distintos.

---

## Phase 7: User Story 5 — secretos locales (P1)

**Goal**: HU5: crear valores ficticios exclusivamente en la bóveda del dispositivo sin exposición ni operaciones diferidas.

**Independent Test**: Crear ficticio, reabrir y comprobar persistencia local y ausencia del valor en UI, DB, logs y contexto; falta de backend y resultados desconocidos fallan cerrados.

Las pruebas indicadas van antes del comportamiento. Tras GREEN, refactorizar sin perder cobertura. Las tareas que implementan modelos conservan sus constraints del diseño.

- [ ] T053 [P] [US5] Escribir pruebas RED del contrato POST secretos en `backend/tests/secrets_contract.rs`: creador autenticado, metadatos de espacio propio, duplicado, error y confirmación sin valor; no existen rutas de revelar/cambiar/borrar/exportar/ejecutar ni acceso por rol administrador. (Requiere T012.)
- [ ] T054 [P] [US5] Escribir pruebas RED de reservas concurrentes, fallo antes/después de bóveda y confirmación indeterminada en `backend/tests/secrets_state.rs`; no falso éxito, overwrite por reintento o valor en SQLite/sesión/logs. (Requiere T012.)
- [ ] T055 [P] [US5] Escribir pruebas RED en `frontend/src/tests/secrets.spec.ts` para vaciar el valor tras cada intento, confirmación/error sin eco, ausencia en localStorage/consola/estado persistido y fallos del backend Windows; solo señuelos ficticios. (Requiere T012.)
- [ ] T056 [P] [US5] Crear SecretReservation en `backend/migrations/0008_secrets.sql` y `backend/src/models.rs`: «Tabla fuera de búsquedas/wiki/eventos de conocimiento. Nombre único por creador, destino opaco único y nunca reutilizado. `state`: reserved, created, indeterminate o failed. No almacena el valor; no ofrece listado global que revele nombres de otros creadores.» (Requiere T053, T054, T055.)
- [ ] T057 [P] [US5] Implementar adaptador aislado `backend/src/windows_vault.rs` con windows-sys, CRED_TYPE_GENERIC y CRED_PERSIST_LOCAL_MACHINE explícitos; validar tamaños/errores/propiedad de buffers/CredFree y encapsular unsafe, sin persistencia enterprise, cifrado casero, fallback o acceso genérico al inventario de credenciales. (Requiere T053, T054, T055.)
- [ ] T058 [US5] Implementar creación en `backend/src/services.rs`: reservar metadatos y cerrar transacción antes de escribir una vez en la bóveda, confirmar después; fallo desconocido conserva indeterminate y se reconcilia localmente sin devolver valor o repetir ciegamente. Reutilizar windows_vault.rs; no asumir atomicidad entre almacenes. (Requiere T056, T057.)
- [ ] T059 [US5] Conectar POST `/api/secrets` en `backend/src/api.rs` con DTO de entrada excluido de Debug/log/error reflection, autorización/CSRF y salida no sensible; denegar backend no disponible, sin almacenar valor en reservas/sesiones/contexto. (Requiere T058.)
- [ ] T060 [P] [US5] Implementar `frontend/src/views/SecretCreateView.vue` con valor transitorio y vaciado después de cualquier intento, sin campos de revelado/actualización ni persistencia del valor; dar GREEN a secrets.spec.ts usando contrato fijado. (Requiere T053, T054, T055.)
- [ ] T061 [US5] Integrar creación en `frontend/src/App.vue` y `frontend/src/api.ts`, reutilizando sesión/CSRF y tipos sin incorporar valores a logs, rutas, mensajes o almacenes generales. (Requiere T059, T060.)
- [ ] T062 [US5] Implementar y ejecutar separadamente `backend/tests/windows_vault.rs` con backend Windows habilitado explícitamente, target ficticio único y verificación de persistencia local tras reapertura; exigir al menos un caso ejecutado y limpieza solo del fixture propio, sin enumerar/borrar credenciales existentes. (Requiere T061.)
- [ ] T063 [US5] Ejecutar secretos/estado/UI y comprobar señuelo ausente de DB, logs, búsquedas US3 y contexto US4 en `backend/tests/secrets_contract.rs`, `backend/tests/secrets_state.rs` y `frontend/src/tests/secrets.spec.ts`; registrar SC-005 y limitación de salida real en `specs/001-sipibrain-collective-memory/tasks.md`. No usar secretos reales ni declarar resuelta la política de pegado accidental. (Requiere T062, T046, T051.)

**Checkpoint**: cerrar solo con evidencia del criterio independiente. Mantener pendientes los pasos externos; conformidad sintética y aceptación real son resultados distintos.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Goal**: Consolidar evidencia honesta sin ampliar alcance ni publicar el sistema.

**Independent Test**: Separar pruebas de documento, código, navegador, bóveda y servicios; no cerrar pruebas bloqueadas por ausencia de entradas externas.

- [ ] T064 Tras los checks de cada historia, ejecutar verificación independiente acotada de los cambios reales de seguridad y concurrencia (items 2 y 4) sobre `backend/src/auth.rs`, `backend/src/access.rs`, `backend/src/services.rs` y `backend/src/windows_vault.rs`; registrar evidencia y correcciones limitadas en `specs/001-sipibrain-collective-memory/tasks.md`, sin inventar PASS, fijar modelos ni activar revisión nativa por cuenta propia. (Requiere T028, T035, T046, T051, T063.)
- [ ] T065 Actualizar `specs/001-sipibrain-collective-memory/quickstart.md` y `specs/001-sipibrain-collective-memory/plan.md` con versiones/lockfiles, comandos realmente entregados y resultados observados; mantener falsos/sintéticos, servicios no disponibles, políticas pendientes y el minuto no aprobado claramente separados. (Requiere T064.)
- [ ] T066 Recorrer todos los escenarios de `specs/001-sipibrain-collective-memory/quickstart.md` sobre backend Rust y Vue reales locales y repetir suites de `backend/Cargo.toml` y `frontend/package.json`; registrar código de salida/número de casos, estado de negocio inmutable tras rechazo y evidencia sanitizada. Si faltan servicios, registrar SC-002 bloqueado sin ejecutar sustitutos. (Requiere T065.)
- [ ] T067 Reconciliar FR-001–FR-013 y SC-001–SC-007 en `specs/001-sipibrain-collective-memory/tasks.md`, conservando sin marcar las tareas reales bloqueadas y declarando cada prueba no ejecutada; registrar responsables/entradas pendientes y próximo paso, sin commits, push, publicación, secretos reales o eliminación fuera de la autorización de implementación. (Requiere T066.)

**Checkpoint**: completar y observar los prerrequisitos antes de depender de ellos; no marcar tareas por la mera existencia de archivos.

---

## Dependencies & Execution Order

### Phase Dependencies

Setup T001–T003 → Foundational T004–T012. T012 bloquea toda implementación de historias. Los números siguen el orden de lectura/ejecución recomendado; las dependencias explícitas gobiernan qué puede avanzar si una tarea externa está bloqueada.

### User Story Dependencies — grafo de aristas

| Nodo | Depende de | Resultado / condición |
|---|---|---|
| US1, T013–T028 | T012 | Núcleo de memoria, autoría, revisión y conflictos; MVP |
| US2 sintética, T029–T035 | T028 | RFD/proyecto/mediación con dobles; no aprueba SC-002 |
| US2 real, T036–T038 | T035 + entradas/servicios autorizados | No frena US3/US5 si está bloqueada |
| US3, T039–T046 | T028 | Búsqueda manual sobre fixtures; no necesita US2 real |
| US4 sintética, T047–T051 | T028 + T035 | Nueva conversación sobre memoria persistente |
| US4 real, T052 | T051 + T038 | SC-002 completo con nueva conversación e intercambio real |
| US5 núcleo, T053–T062 | T012 | Implementable sin agentes reales; bóveda requiere prueba Windows explícita |
| US5 comprobación cruzada, T063 | T062 + T046 + T051 | Ausencia de señuelo en búsquedas/contexto; no sustituye inspección de salida real |
| Cierre, T064–T067 | Núcleos y checks sintéticos observados | Si T036–T038/T052 faltan, documentar parcial y conservar esas casillas sin marcar |

### Within Each User Story

Pruebas → modelos/constraints → servicios compartidos → interfaces → integración → criterio independiente. Modificaciones de `backend/src/models.rs`, `backend/src/services.rs`, `backend/src/api.rs`, migraciones, manifests y `frontend/src/api.ts`/`types.ts` se serializan bajo un único responsable a la vez. No se duplican funciones de guardado, acceso, recibos o secretos por canal.

**Bloqueos externos actuales**: T036, T037, T038 y T052 no tienen sus entradas reales disponibles. Obtener contrato/destino/puerto/identidad y autorización no significa descubrirlos por escaneo o reutilizar sesiones ambientales. Un mediador ausente/denegado produce cero contenido ajeno; un mock o puerto abierto no acredita SC-002. Solo adaptar protocolos suministrados. Un código cero con cero casos ejecutados no cierra pruebas ignoradas.

**Políticas diferidas**: respaldo, uso/permisos de operaciones de secretos no incluidas, protección frente a entrada accidental en notas y datos reales. No se «resuelven» implementando una ruta adicional. El minuto propuesto requiere dispositivo/volumen acordados y no es un umbral aprobado.

### Parallel Opportunities

Lotes válidos tras satisfacer **todos** los prerrequisitos del lote; el resto de tareas se considera secuencial si comparte archivos:

| Lote | Tareas | Superficies disjuntas |
|---|---|---|
| Inicialización | T001 + T002 | backend frente a frontend; ninguna edita configuración raíz compartida |
| Seguridad base | T004 + T005 | security.rs frente a auth.spec.ts |
| US1 pruebas | T013 + T014 + T015 + T016 | Cuatro archivos de pruebas distintos, helpers de T003 solo lectura |
| US1 vistas | T024 + T025 | WikiView/RecordEditor frente a ConflictsView; wiring compartido en T026 |
| US2 pruebas | T029 + T030 | exchange_contract.rs frente a project_scope.rs |
| US3 pruebas/vistas | T039 + T040; después T043 + T044 | Backend/frontend; vistas distintas después de T042, wiring en T045 |
| US4 pruebas | T047 + T048 | continuity.rs frente a agent_identity.rs |
| US5 pruebas | T053 + T054 + T055 | Contrato, estados y Vue en archivos distintos |
| US5 implementación disjunta | T056 + T057 + T060 | Esquema/modelo, wrapper Windows y vista; solo después de T053–T055 y sin otra edición de models.rs |

Esto no permite poner historias completas en paralelo indiscriminadamente: sus servicios/rutas/modelos se comparten. La ejecución paralela de escritores requiere superficies declaradas y un responsable de integración, según AGENTS.md.

## Parallel Example: User Story 1

Tras T012: T013/T014/T015/T016 escriben pruebas en archivos diferentes. Tras T023: T024/T025 trabajan vistas disjuntas; T026 integra App/API y mantenimiento, de forma secuencial.

## Parallel Example: User Story 2

Tras T028: T029 y T030 cubren contrato y ámbito con fixtures distintos. T031–T035 comparten modelo/servicios/adaptadores y se ejecutan en orden. No iniciar llamadas reales para «paralelizar» una autorización pendiente.

## Parallel Example: User Story 3

Tras T028: T039/T040 pueden preparar las dos pruebas; tras T042, T043/T044 implementan vistas aisladas. T045 es dueño del wiring compartido.

## Parallel Example: User Story 4

Tras T035: T047/T048 prueban continuidad e identidad en archivos distintos. T049/T050 se serializan por compartir integrations.rs. T052 sigue condicionado a entradas y prueba real de US2.

## Parallel Example: User Story 5

Tras T012: T053/T054/T055. Tras completar esas pruebas y fijar tipos/contratos: T056/T057/T060 son disjuntas; T058/T059/T061 integran servicio/API/cliente. T062 exige target ficticio exclusivo y prueba Windows consciente, no escaneo de la bóveda.

## Implementation Strategy

### MVP First — US1

Completar T001–T028: infraestructura, identidad/asignaciones, fuentes y memoria por agente, referencias/revisión, autoría, dos propuestas y resolución administrativa. Validar persistencia, aislamiento y flujo navegador/API con datos sintéticos. El MVP no incorpora por adelantado las operaciones diferidas de secretos ni afirma interacción real entre agentes.

### Incremental Delivery

Después del MVP, US2 sintética y US3 amplían recuperación/consulta; US4 demuestra otra conversación; US5 añade creación local ficticia. Las dependencias permiten avanzar US3/US5 sin disponer de servicios reales, pero conservan las integraciones compartidas bajo un único escritor. Ejecutar T036–T038 y T052 solo cuando estén disponibles y autorizadas sus entradas; nunca marcar completitud por sustitución.

### Completion Evidence

Cada casilla requiere resultado observado y comandos/casos ejecutados, o permanece abierta. Eliminar solo fixtures temporales de esta ejecución, conservar fuentes y propuestas y registrar intentos denegados sin valores sensibles. Las suites ordinarias no usan servicios ni bóveda reales. Una prueba de componente no es e2e.

La generación de este archivo no autoriza implementar ni publicar. El próximo comando sugerido es `$speckit-analyze` para revisar consistencia o `$speckit-implement` cuando se solicite ejecución. No crear otra feature ni reformular requisitos de producto como atajo.

## Notes

Resumen: 67 tareas; Setup 3, Foundational 9, US1 16, US2 10, US3 8, US4 6, US5 11 y cierre 4. Todas las historias son P1; el orden no cambia sus prioridades.

La documentación de tecnologías sigue AGENTS.md y el plan Rust/Vue vigente. Las notas del AGENTS.md que describen el antiguo estado del plan no reintroducen el stack anterior. La referencia a `specs/README.md` continúa sin archivo disponible; no se crea un estado ficticio para suplirla.

No hay aplicación, pruebas ejecutadas, Git local ni autorización de publicación por esta generación. Las verificaciones de este comando solo cubren formato y consistencia de tareas.

Corrección documental C1/C2: T004/T008/T012 asignan pruebas RED, implementación serve/listener/bundle Vue y smoke con build real antes del MVP. T003 completa el contrato project_id antes de T039–T046, que implementan y prueban consulta general por proyecto para todos los tipos. IDs, dependencias, casillas, conteos e hitos externos se conservan; esta corrección no ejecuta esas tareas ni modifica por sí misma los contratos o el código.

## Evidencia de implementación MVP

- T001: cargo check --manifest-path backend/Cargo.toml y cargo fmt observados correctos (Rust 1.98.0; Argon2 0.6.0 estable; SQLx/sesiones compatibles).
- T002: npm run typecheck, npm test -- --run (1 prueba de montaje) y npm run build correctos. Typecheck repetido por el integrador, correcto. Setup sin RED funcional; aún no acredita seguridad ni flujo con backend.

- T003: contrato JSON, DTO Rust/TypeScript, q/project_id y harness aislados; cargo test --test dto: 1 PASS; typecheck PASS. Interfaz local de revisión/mantenimiento documentada antes de exposición.
- T004: RED real: security 6 aserciones (501 vs 200/401/403); server_startup 2 aserciones (sin listener/directorio inválido aceptado). Runner/compilación funcionales.
- T005: RED real: auth.spec.ts 4 aserciones (token prelogin/login, error seguro y formulario accesible). Todavía no GREEN.

- T006–T009: cargo check/clippy --all-targets -D warnings y cargo test --locked correctos: 11 pruebas (DTO1/seguridad7/arranque3). Security7 repetidas por integrador: PASS. Sesiones reabiertas/rotadas/invalidadas, Argon2 PHC, CSRF+Origin, asignaciones, listener/static y API JSON probados. Sin servicios externos. T012 aún pendiente de Vue integrado.

- T010–T011: RED 6 aserciones observado; GREEN typecheck/Vitest 7/7 y build Vue. Cliente cookie+CSRF en memoria y mensajes fijos; login y gestión de asignaciones accesibles. T012 smoke real en curso, todavía no acredita flujo navegador.

- T012: cargo check/test --locked 11 PASS; Vue typecheck/Vitest7/build PASS. Serve real del bundle: raíz200, JS real /assets/index-0xuW8vOp.js 200 (75152 bytes), API desconocida404 JSON; child detenido, SQLite en temp fuera del workspace.
- T013–T015: RED ejecutado (persistencia3/conflictos2/contrato2), creación de fuente404 vs201 y registro405 vs201. Test runner funcional; aserciones profundas de inmutabilidad/conflicto todavía no acreditadas hasta GREEN. Base11 sigue PASS.

- T016: typecheck PASS y wiki.spec.ts RED 5 aserciones funcionales (aportación/procedencia/revisión, candidatas pendientes, resolución409, retención503, propuesta202); placeholder no integrado, GREEN pendiente tras T023.
