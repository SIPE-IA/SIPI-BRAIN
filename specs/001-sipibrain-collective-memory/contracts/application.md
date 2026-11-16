# Application Contract: API Rust e interfaz Vue 3 + TypeScript

Contrato propuesto para la aplicación por construir conforme a [AGENTS.md](../../../AGENTS.md). Las rutas `/api` son de SIPI-brain, **no endpoints existentes de Insight o Estimador**. Sus adaptadores se definen en [agent-exchange.md](agent-exchange.md).

## Frontera web y autenticación

- Backend Rust/Axum ligado a loopback; sirve la API y el bundle Vue compilado bajo el mismo origen. Desarrollo: Vite usa proxy `/api` al backend configurado. No CORS general ni exposición en LAN; acceso remoto requiere otra autorización.
- Usuarios persistidos con hash Argon2id/PHC de la biblioteca y salt seguro. Administrador inicial y usuarios de prueba se crean por comando interactivo, sin contraseña predeterminada ni password en argumentos.
- Sesiones tower-sessions persistidas mediante SQLx/SQLite, sin valores del almacén de secretos. Rotar ID al login y eliminar sesión al logout. Cookies HttpOnly, host-only, SameSite y expiración explícitos; Secure para HTTPS. La excepción HTTP local es solo de desarrollo sintético, no un ajuste para publicar datos reales.
- Token CSRF aleatorio ligado a sesión y validación de Origin exacto para mutaciones, incluido login/logout. Un GET de sesión puede iniciar el estado preautenticado y entregar un token CSRF; después del login se rota la sesión y se obtiene el token vigente. No construir algoritmos de sesión o hashing propios.
- Autenticación, rol y WikiAssignment se validan en el backend en toda operación. El frontend no envía un rol confiable y sus controles de navegación no otorgan permisos.
- Administrar asignaciones no permite leer contenido fuera de las wikis asignadas. Las llamadas de agentes requieren identidad verificada por un adaptador autorizado, nunca el nombre recibido en JSON como autenticación. Sin ese canal, el adaptador real está deshabilitado.
- No telemetría externa ni registro de cuerpos sensibles. Vue muestra texto escapado; no `v-html` para conocimiento arbitrario. Sesiones y valores secretos no se guardan en localStorage.

## Operaciones propuestas

JSON tipado mediante serde en la API y DTO TypeScript en el cliente. El navegador envía la cookie y el encabezado CSRF en mutaciones. La lógica de acceso, guardado y resolución se reutiliza en servicios Rust; no se duplica entre UI y agentes.

| Operación | Entrada | Autorización | Resultado |
|---|---|---|---|
| `GET /api/session` | Cookie de sesión | Estado propio, incluso prelogin | Usuario actual o null y token CSRF; sin valores secretos |
| `POST /api/login`, `POST /api/logout` | Credenciales o sesión, CSRF y Origin | Usuario activo / sesión válida | Login con rotación o logout con invalidación; fallo genérico sin eco de contraseña |
| `GET /api/wikis` | Sesión | Autenticado | Solo wikis asignadas, sin títulos/conteos de wikis ajenas |
| `GET /api/wikis/{wiki_id}/records` | Texto, agente o cliente, paginación | Asignación | Índice y resultados del ámbito; ausencia explícita |
| `GET /api/wikis/{wiki_id}/records/{record_id}` | Identificadores | Asignación y pertenencia record/wiki | Contenido, proyecto/cliente, origen, referencias, revisión y estado |
| `POST /api/wikis/{wiki_id}/sources` | Fuente seleccionada y contexto | Asignación | Fuente original inmutable; no actualización encubierta de fuente ajena |
| `POST /api/wikis/{wiki_id}/records` | Tipo, título, contenido, proyecto, referencias y submission_id | Asignación | Registro; RFD exige proyecto, no plantilla documental arbitraria |
| `POST /api/wikis/{wiki_id}/records/{record_id}/proposals` | Base, submission_id y propuesta | Asignación al guardar | Guardado normal o propuesta conservada en conflicto |
| `GET /api/wikis/{wiki_id}/conflicts` | Sesión | Asignación | Casos pendientes, separados de conocimiento vigente |
| `POST /api/wikis/{wiki_id}/conflicts/{conflict_id}/resolve` | Generación, contenido decidido y motivo | Administrador y asignación | Resolución manual trazable sin destruir propuestas |
| `GET /api/admin/assignments`, `POST /api/admin/assignments` | Usuario/wiki, operación de asignación | Administrador | Gestión de asignaciones sin acceso lateral al contenido o a secretos |
| `POST /api/secrets` | Nombre y valor | Programador autenticado, espacio propio de creación | Reserva y creación local; confirmación no sensible, sin valor en salida |

La página Vue se sirve en `/`; vistas de login, wikis, registro, conflictos, asignaciones y creación de secretos consumen las operaciones anteriores. No se requieren una biblioteca global de estado o un segundo backend. Los registros, fuentes y revisiones no se exponen mediante un CRUD administrativo genérico que eluda las reglas.

## Guardado y conflictos

- Misma intención identificada por autor, operación y submission_id: mismo contenido devuelve el registro ya confirmado, sin duplicarlo. Reutilizar el ID con contenido distinto devuelve `409 idempotency_conflict`.
- En escritura normal responder `201`; una propuesta preservada en conflicto puede responder `202` con `state=pending`. No confundir conflicto preservado con pérdida de guardado. Confirmación solo después del commit.
- Primero sin conflicto puede avanzar el head. Si aparece una segunda propuesta desde la base anterior, conservar ambas y suspender la presentación de una ganadora definitiva. Resolver requiere administrador y generación actual.
- Una resolución desactualizada responde `409 conflict_changed`; ninguna candidata recién llegada se descarta. Un no administrador no puede asignarse wikis ni resolver un caso.
- SQLite/SQLx utiliza transacciones cortas sobre una conexión retenida, sin llamadas externas dentro. Ante espera agotada informar no guardado y conservar solo el texto ordinario del formulario para reintento.

## Errores y protección de datos

Formato de error JSON estable: código no sensible y mensaje para la UI, sin reflejar cuerpos de entrada.

| Condición | Respuesta | Invariante |
|---|---|---|
| Sin sesión o credencial inválida | `401 unauthenticated` | Sin conocimiento ni identidad ajena |
| CSRF/Origin inválido | `403 invalid_request_origin` | Sin mutación |
| Wiki/registro ajeno o inexistente | `404 not_found` genérico | Sin revelar existencia, títulos o conteos |
| No administrador asigna/resuelve | `403 forbidden` | Permisos/propuestas intactos |
| Campos o asociaciones incoherentes | `400 invalid_input` | Sin escritura parcial |
| Reintento incompatible o resolución obsoleta | `409` | Sin sobrescritura ni descarte |
| Bloqueo persistente de almacenamiento | `503 storage_busy` | No afirmar commit |
| Bóveda Windows no disponible | `503 local_vault_unavailable` | Sin fallback de almacenamiento |
| Nombre propio de secreto ya reservado | `409 secret_name_unavailable` | No sobrescribir ni repetir a ciegas |
| Resultado de bóveda desconocido | `creation_indeterminate` sin valor | No afirmar éxito ni reescribir automáticamente |

El cliente vacía el campo de secreto después del intento; no lo persiste ni lo escribe en consola. El backend excluye esos DTO de logs/Debug/errores con cuerpo. Una respuesta de validación no devuelve el valor; una página de error tampoco.

## Secretos: frontera Windows explícita

Adaptador Rust windows-sys con CRED_TYPE_GENERIC y CRED_PERSIST_LOCAL_MACHINE; [estados y metadatos](../data-model.md). Solo nombre/reserva y destino opaco en SQLite, sin valor. Reserva y escritura Windows no son atómicas: conservar indeterminate si no se conoce la confirmación. No hay rutas de revelar, modificar, borrar, exportar, ejecutar o entregar valores a agentes. Un rol administrativo o asignación de wiki no crea esas capacidades.

El wrapper de la API Windows controla tamaños, errores, propiedad de buffers y CredFree. No usar persistencia empresarial ni backend alternativo. La cuenta del sistema operativo sigue siendo frontera de confianza; el diseño no protege contra software con los mismos privilegios Windows. Valores ficticios mientras respaldo y prevención de pegado accidental sigan pendientes.

## Comandos de aplicación a implementar

El futuro binario `sipibrain` expondrá `init-db`, `bootstrap-admin`, `create-user --username <nombre>`, `seed-pilot --synthetic` y `serve`. Contraseñas de bootstrap/usuario se leen interactivamente, nunca en argumentos. Sembrar datos no crea contraseñas conocidas ni sobrescribe aportaciones existentes. Flags de servicio: `--bind` loopback y `--static-dir` del bundle compilado.

Estos son contratos propuestos, no comandos existentes. Cargo compila/prueba el backend; npm/Vite compilan el frontend. La [guía](../quickstart.md) distingue futuras ejecuciones de resultados comprobados.
