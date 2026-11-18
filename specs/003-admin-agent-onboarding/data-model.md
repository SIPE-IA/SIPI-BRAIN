# Modelo de datos: alta de agentes y prueba de conexión

| Entidad | Campos y relaciones | Reglas |
| --- | --- | --- |
| Agente (existente) | `id`, `name`, `active`; cero o una wiki y cero o una configuración de conexión. | Identificador y nombre no vacíos tras quitar espacios exteriores. Alta activa. PK existente; un duplicado no cambia la fila. No incorporar otra operación de edición o baja. |
| Wiki (existente) | `id`, `name`, `agent_id`. | FK y unicidad sobre `agent_id`; solo agente activo elegible. No depende de conexión y no crea asignaciones. |
| Configuración de conexión (nueva) | `agent_id` PK/FK, `destination_id`, `expected_identity`, `revision` opaca. | Identidad no vacía, compatible con el contrato de identidad. Guardar solo una clave de catálogo aprobada y compatible. Generar revisión nueva en cada cambio efectivo en la misma escritura atómica; guardar los mismos valores no invalida. No guardar URL, secreto ni copia de la aprobación. |
| Destino aprobado (configuración del entorno) | Clave y etiqueta no sensible, alcance del destino efectivo, contrato/adaptador e identidad verificable; canal autorizado si aplica. | Fuera de administración web. La aprobación es autoritativa; un destino retirado puede seguir referenciado en SQLite, pero no se puede volver a seleccionar ni contactar. No enviar datos internos ni autenticación al navegador. |
| Resultado de prueba (efímero) | Revisión probada y código cerrado: `verified`, `unreachable`, `identity_unverified`, `not_testable`. | Fuera de SQLite. No incluir respuestas remotas. Solo es aplicable a la revisión y aprobación vigentes; el panel no restaura resultados tras recargar. |

## Estados observables

- **Agente registrado → wiki creada**: operaciones independientes. Rechazar duplicados, agente inactivo o segunda wiki sin alterar lo existente.
- **Sin configuración → configurada → modificada**: la primera escritura crea la revisión; cada cambio efectivo sustituye la revisión anterior. Revocar el destino deja la configuración guardada pero no disponible para pruebas.
- **Sin resultado → prueba en curso → resultado**: el backend verifica aprobación antes de contactar y revalida configuración y aprobación antes de confirmar éxito. El panel borra el resultado al cambiar la configuración y descarta respuestas desplazadas por una revisión o solicitud nueva. Un fallo no muta agente, wiki ni asignaciones.

Permisos, fuentes, registros e historial conservan el modelo de la 001 y la proyección administrativa de la 002.
