# Contrato propuesto: incorporación de agentes

Diseño para la API local; no describe endpoints de un agente SIPI real. Todas las operaciones son administrativas, con sesión vigente, rol validado en el servidor y `Cache-Control: no-store`. Para operaciones mutables y la prueba explícita se exige el flujo existente de origen y CSRF. Validar autenticación/autorización antes de leer configuración privilegiada o iniciar contactos.

| Operación | Entrada | Salida |
| --- | --- | --- |
| `GET /api/admin/agents` | Ninguna. | `200 { items: AgentOnboardingItem[] }`. Incluye agentes con y sin wiki. |
| `POST /api/admin/agents` | `{ id, name }`. Los IDs `.` y `..` están reservados para navegación de rutas y se rechazan tras quitar espacios exteriores. | `201 AgentOnboardingItem` después de confirmar la escritura. |
| `POST /api/admin/wikis` | `{ name, agent_id }`. | Sin cambios: `201 WikiAdminItem` conforme al [contrato 002](../../002-wiki-admin-monitoring/contracts/wiki-admin.md). |
| `GET /api/admin/connection-destinations` | Ninguna. | `200 { items: { id, label }[] }` con destinos aprobados y compatibles. Una lista vacía es válida. |
| `PUT /api/admin/agents/{agent_id}/connection` | `{ destination_id, expected_identity }`. | `200 ConnectionSummary` solo tras guardar; no contacta el destino. |
| `POST /api/admin/agents/{agent_id}/connection/test` | `{ revision }`, obtenida de la configuración guardada. | `200 { status, revision }`; para una configuración ausente o distinta, `not_testable` y revisión solicitada, sin contacto. |

`AgentOnboardingItem`: `id`, `name`, `active`, `connection: ConnectionSummary | null`.
`ConnectionSummary`: `destination_id`, `expected_identity`, `revision`. No enviar URL, secretos, referencia al almacén ni cuerpos remotos. El catálogo de wikis conserva el selector de agentes elegibles.

## Semántica de errores y resultados

- Errores de solicitud: formato de error existente; `401 unauthenticated`, `403 forbidden` para rol insuficiente, `403 invalid_request_origin` para origen o CSRF inválidos, `400 invalid_input` para campos inválidos, destino no aprobado o incompatible o agente inválido (sin contacto), `409 agent_already_exists` para duplicados. Mantener los códigos de la 002 para la wiki. Un fallo de almacenamiento no devuelve un éxito ni un resultado simulado.
- `verified`: identidad comprobada conforme al contrato aprobado y coincide exactamente con la esperada; configuración y aprobación siguen correspondiendo a la prueba. `unreachable`: error de transporte o plazo agotado. `identity_unverified`: servicio alcanzado sin prueba de identidad válida o coincidente. `not_testable`: configuración ausente, revisión distinta o desplazada, destino revocado/incompatible, o intento de redirección rechazado. No usar un resultado de transporte como señal de identidad.
- La revocación es efectiva al activar de forma atómica la nueva versión del catálogo en el backend en ejecución, sin reinicio. Desde la activación, `GET /api/admin/connection-destinations` omite el destino y `PUT /api/admin/agents/{agent_id}/connection` rechaza seleccionarlo con `400 invalid_input`. La configuración ya guardada se conserva. La prueba devuelve `200` con `not_testable` y la revisión de configuración solicitada: no inicia contacto si la revocación ya era efectiva y no confirma éxito si la revocación ocurre durante el contacto antes de confirmar el resultado. Una reaprobación con la misma clave recibe una nueva versión de aprobación y no convalida la prueba anterior. Si no se puede comprobar la aprobación vigente, la prueba no inicia contacto ni confirma éxito.
- El servidor no sigue redirecciones, no descubre servicios y no permite que la solicitud reemplace el destino o el mecanismo de autenticación. Solo el adaptador autorizado puede contactar el destino efectivo, con plazo total y sin leer o transferir contenido. Resultado y trazas contienen únicamente metadatos seguros, nunca cuerpos ni credenciales.

## Panel

Ofrecer **Register agent**, reutilizar **Create wiki** y mostrar el selector de destinos aprobados y **Test connection** para la configuración guardada. Tras un alta confirmada, refrescar el listado y la elegibilidad de wikis; si falla el refresco, distinguirlo del resultado del alta. Sin configuración o destinos compatibles, mostrar un estado claro sin bloquear el alta ni la creación de la wiki. No reutilizar el resultado si la vista cambia de agente/revisión o una respuesta llega fuera de orden.
