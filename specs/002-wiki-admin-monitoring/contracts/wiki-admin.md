# Contrato propuesto: administración de LLMwikis

Fase de diseño; las rutas nuevas no se presentan como implementadas. Usar sesión vigente y formato de errores de la aplicación. El servidor comprueba el rol administrativo activo en las tres operaciones. Responder sin caché compartida o del cliente (`Cache-Control: no-store`) para no reutilizar una proyección con autoría después de cambios de permisos.

| Operación | Entrada | Respuesta de éxito |
| --- | --- | --- |
| `GET /api/admin/wikis` | Sin filtros. | `200`: `{ items: WikiAdminItem[], eligible_agents: AgentSummary[] }`. Incluye todas las wikis y solo agentes activos sin wiki en el selector. Un selector vacío es válido. |
| `POST /api/admin/wikis` | `{ name, agent_id }`; origen admitido y token CSRF conforme al flujo existente. | `201` + `WikiAdminItem`, únicamente después del commit. |
| `GET /api/admin/wiki-activity` | Sin filtros. | `200`: `{ items: WikiActivitySummary[] }`, una entrada por cada wiki, incluso sin asignación de lectura. |

`AgentSummary`: `id`, `name`.  
`WikiAdminItem`: `id`, `name`, `agent: AgentSummary`.  
`WikiActivitySummary`: campos de `WikiAdminItem` más `record_count`, `source_count`, `last_contribution_at` (UTC RFC 3339 o `null`), `recent_activity`. Si el administrador tiene una asignación de lectura vigente a esa wiki, cada entrada de actividad DEBE incluir `author_name`, el nombre de usuario histórico asociado al evento. Sin esa asignación, la respuesta DEBE omitir `author_name` y cualquier identificador, etiqueta u otro dato que revele la autoría. En ambos casos, cada entrada DEBE incluir `occurred_at` (UTC RFC 3339). No enviar cuerpos, acciones internas, referencias de eventos, secretos ni referencias a secretos.

Los detalles, fuentes e historial completo conservan sus rutas y guardas de asignación existentes. El listado administrativo y la proyección no constituyen un permiso de lectura de contenido. Evaluar la autorización con datos vigentes del servidor, nunca con una asignación o un rol recibido del cliente.

## Errores

- `401 unauthenticated` sin sesión; `403 forbidden` sin rol administrativo.
- En el alta: `403 invalid_request_origin` si el origen o token no es válido, `400 invalid_input` para nombre vacío o agente inexistente/inactivo y `409 wiki_already_exists` si el agente ya tiene wiki. Un error de almacenamiento usa códigos existentes de la aplicación y no confirma éxito.
- Un fallo de supervisión devuelve un error explícito, no datos parciales inventados ni `200` con contadores en cero. En rutas existentes mantener los códigos actuales de denegación.

## Interfaz

- Mostrar la entrada «Manage wikis» al administrador, con opción «Create wiki», nombre y selector de agente. Si no hay candidatos, indicar que no hay agentes elegibles. No ofrecer creación de agentes.
- Tras `201`, reflejar la wiki guardada y actualizar el catálogo. Si el refresco falla después de confirmar, distinguir ese fallo del resultado del alta. No insertar filas optimistas ni anunciar éxito sin `201` válido.
- Mostrar carga, error y vacío como estados distintos. Actualizar bajo demanda. Mostrar las fechas de las wikis no asignadas sin huecos que se interpreten como ausencia de actividad. Los nombres del autor solo aparecen cuando vienen autorizados en la respuesta.
