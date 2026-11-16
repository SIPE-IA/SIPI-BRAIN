# Data Model: SIPI-brain

Diseño propuesto para backend Rust/SQLx y frontend Vue 3 + TypeScript, no esquema instalado. Las entidades deben satisfacer [spec.md](spec.md), [research.md](research.md) y los contratos de [aplicación](contracts/application.md) e [intercambio](contracts/agent-exchange.md).

## Convenciones e invariantes

- Identificadores opacos estables; los ejemplos RFD-001…RFD-004 no imponen una numeración de negocio.
- Tiempos UTC; mostrar fecha local sin cambiar el orden cronológico persistido.
- Todo conocimiento pertenece a una única `wiki_id`. No aceptar esa pertenencia, la autoría o el rol administrativo solo porque aparezcan en el cuerpo de una solicitud.
- Separar datos del negocio de identidades de autenticación. Los valores de secretos nunca aparecen en las tablas siguientes.
- Fuentes y contenidos de revisiones/propuestas son inmutables. Las transiciones de estado se registran, no destruyen esos contenidos. No hay borrado de fuentes ni restauración general de revisiones por interfaz.

## Entidades

| Entidad | Campos principales | Relaciones y validaciones |
|---|---|---|
| `User` | `id`, nombre de usuario único, hash PHC Argon2id, estado activo y rol | Backend Rust autentica y comprueba el rol; sin contraseñas predeterminadas ni rol recibido del navegador. El hash de contraseña no es un valor del almacén de secretos de usuario. |
| `Session` | Identificador aleatorio de la biblioteca, usuario, expiración y estado CSRF | Sesión persistente de tower-sessions/SQLx; rotar al login e invalidar al logout. Sin valores secretos, contraseñas o roles de confianza copiados del cliente. |
| `Agent` | `id`, `name`, estado activo | Insight y Estimador son las identidades iniciales de prueba. No se confía en un nombre recibido para autenticar un agente. |
| `Wiki` | `id`, `agent_id`, `name` | Unicidad de `agent_id`: una wiki propia por agente. |
| `WikiAssignment` | `user_id`, `wiki_id`, `assigned_by`, `assigned_at` | Par usuario/wiki único; solo administrador gestiona asignaciones. Lectura, búsquedas y aportaciones siempre requieren esta relación. Administrar asignaciones no da acceso al contenido por una ruta alternativa. |
| `Client` | `id`, `display_name` | El nombre no es identificador único; admite clientes homónimos sin mezclarlos. |
| `Project` | `id`, `name`, `client_id` opcional | Proyecto identificable incluso sin cliente externo. Cliente no se deduce por similitud de nombres. |
| `OriginalSource` | `id`, `wiki_id`, `body`, `content_hash`, `origin_kind`, `origin_ref`, `contributor_id` o `agent_id`, `project_id` opcional, `received_at` | Original inmutable, seleccionado para ingestión, no captura indiscriminada de conversaciones. No se actualiza mediante síntesis. El hash comprueba integridad, no veracidad. |
| `Record` | `id`, `wiki_id`, `kind`, `title`, `project_id` opcional, `client_id` opcional, `head_revision_id`, `generation`, `state` | `kind`: prompt, decision, note o rfd. Un RFD es un requerimiento funcional y exige proyecto. Si existen proyecto y cliente explícito, deben coincidir. `state`: active o conflicted. |
| `Revision` | `id`, `record_id`, `parent_revision_id` opcional, `body_markdown`, `author_principal`, `created_at`, `epistemic_kind` | Contenido inmutable. Distinguir hecho verificado, decisión y suposición; no convertir una categoría en otra por recuperarla. No ofrece una operación de restauración. |
| `SourceReference` | `revision_id`, `source_id`, localizador opcional | Solo fuentes de la misma wiki. Una copia recibida por intercambio autorizado se registra como fuente local con procedencia y recibo; no se crea una referencia que permita saltar directamente a una wiki ajena. |
| `RecordLink` | Registro de origen, registro de destino, relación | Interrelación explícita dentro de la misma wiki y sujeta a asignaciones; no concede acceso transversal. |
| `EditProposal` | `id`, `record_id`, `base_revision_id`, `proposed_revision_id`, `submission_id`, autor, fecha, estado | UUID de envío único por autor/operación; mismas claves con contenido diferente se rechazan. Base y propuesta deben pertenecer al mismo registro y wiki. El contenido se conserva aunque se detecte conflicto. |
| `EditConflict` | `id`, `record_id`, `base_revision_id`, propuestas candidatas, generación, estado | A lo sumo un conflicto abierto por registro; dos o más propuestas conservadas. La generación cambia si llega otra aportación. Estado pending o resolved. |
| `ConflictResolution` | `conflict_id`, `expected_generation`, `result_revision_id`, `administrator_id`, fecha y motivo | Resolución manual autorizada y atómica; conserva las propuestas y la decisión. No modifica el texto de una fuente original. |
| `WikiMaintenance` | `wiki_id`, reglas no ejecutables de revisión, estado y fecha de revisión | Revisión humana de referencias, contradicciones y obsolescencia. No ejecuta instrucciones recuperadas ni crea un agente de limpieza. |
| `KnowledgeReview` | `revision_id`, resultado de revisión, revisor autorizado, fecha, observación | La consulta muestra revisión y procedencia; algo sin verificar no se presenta como hecho confirmado. Si cambia la revisión, no hereda silenciosamente la revisión humana anterior. |
| `WikiEvent` | `id`, `wiki_id`, secuencia/fecha, actor, acción, referencias y resultado | Registro cronológico de metadatos; sin valores secretos, cuerpos de solicitudes, contraseñas o tokens. Los eventos de una acción rechazada no revelan contenido ajeno. |
| `ExchangeReceipt` | solicitante, wiki origen, destinatario, propósito, proyecto/cliente, referencias solicitadas/permitidas, decisión, identificador del mediador, fecha y vencimiento, estado | Decisión verificable vinculada exactamente a solicitud e identidades. No acepta una bandera `approved` enviada por el solicitante como prueba. Registra referencias, no contenido sensible ni credenciales. |
| `AgentEndpoint` | `agent_id`, destino, puerto, protocolo, referencia al contrato/autorización de prueba | Solo configuración explícita de un responsable. Nada de descubrimiento automático, credenciales ambientales o rutas supuestas. Una referencia no contiene un valor secreto. |
| `SecretReservation` | `id`, `creator_id`, `normalized_name`, `credential_target`, `state`, fechas | Tabla fuera de búsquedas/wiki/eventos de conocimiento. Nombre único por creador, destino opaco único y nunca reutilizado. `state`: reserved, created, indeterminate o failed. No almacena el valor; no ofrece listado global que revele nombres de otros creadores. |

El índice de cada wiki es una consulta sobre sus registros y enlaces autorizados, no otro almacén que deba sincronizarse. No hay base vectorial en el piloto.

## Guardado y concurrencia

1. El cliente envía `submission_id`, la revisión base que leyó y su propuesta. El servidor obtiene autor y permisos de la sesión o canal autenticado; valida asignación dentro de la transacción.
2. En una transacción corta `IMMEDIATE`, persistir la propuesta y su revisión inmutables. Si el ID ya existe con el mismo autor, destino y contenido, devolver el resultado existente; un reintento no crea una tercera propuesta.
3. Si la base coincide con la revisión vigente y no hay conflicto, avanzar el head y la generación. No se exige revisión administrativa para cada edición ordinaria.
4. Si llegó otra edición desde esa base antigua, conservar ambas ramas, abrir el conflicto y marcar el registro `conflicted`. El guardado previo deja de presentarse como ganador definitivo mientras el caso está pendiente. La base se muestra como antecedente, no como selección automática de una candidata.
5. Si ya hay un conflicto abierto, incorporar nuevas propuestas sin elegir ganador y aumentar la generación; el administrador debe revisar el conjunto actualizado. Una aportación tardía tras una resolución previa no borra esa decisión: abre un nuevo caso cuando corresponda.
6. Solo un administrador con acceso a la wiki puede resolver contra `expected_generation`. Una resolución obsoleta no descarta una propuesta recién llegada. Guardar la revisión resultante, resolución y evento en la misma transacción; volver a `active` conservando el caso y sus candidatas.

Las transacciones no abarcan formularios abiertos, decisiones humanas, bóveda ni llamadas de red. Ante bloqueo SQLite, reintentar de forma acotada la operación completa con el mismo ID; si no se confirma el commit, mostrar «no guardado» y conservar el texto en la interfaz. Un error de autorización nunca cambia propuestas, head o generación. Las pruebas concurrentes Rust usan SQLite en archivo y conexiones SQLx independientes; no una única conexión ni una base en memoria compartida como sustituto de esa prueba.

## Fuentes, contexto y revisión

La síntesis crea revisiones con referencias sin sobrescribir originales. Un registro contiene datos, no autoridad: texto que solicite ampliar permisos o revelar secretos no modifica el acceso. La recuperación devuelve contenido con procedencia, categoría y estado de revisión; las propuestas pendientes se identifican como tales y no como contenido vigente. La revisión de vigencia antes de reutilizar conocimiento es humana/contextual, no una nueva función de SIPI NEXUS ni una garantía de verdad.

## Transiciones de secretos

`sin reserva → reserved → created`; si hay fallo seguro antes de escribir, `failed`; si se desconoce el resultado de la bóveda o de la confirmación, `indeterminate`.

Reservar metadatos en SQLite, cerrar la transacción, escribir una vez mediante el adaptador Rust windows-sys con CRED_PERSIST_LOCAL_MACHINE y confirmar metadatos. No existe transacción distribuida. Un estado indeterminado no autoriza sobrescribir ni volver a pedir el valor para repetir ciegamente; exige comprobación local controlada de existencia y estado, sin devolver el valor. Ningún administrador del piloto obtiene por ello una operación pública de revelar o borrar secretos.

Pruebas y almacenamiento inicial usan valores ficticios. La protección frente a pegado accidental en notas, el respaldo y el uso de información sensible real siguen fuera de decisiones aprobadas.
