# Modelo de datos: alta y supervisión

Se reutiliza el esquema SQLite existente. La supervisión es una proyección, no un nuevo registro persistido.

| Entidad | Identidad y relaciones | Reglas de la 002 |
| --- | --- | --- |
| Agente (`agents`) | `id`, `name`, `active`. | Solo un agente activo y sin wiki es elegible para crear una. |
| LLMwiki (`wikis`) | `id` opaco generado por el sistema; `agent_id` único, FK a agente; `name`. | Nombre no vacío tras recortar espacios. La unicidad es por agente, no por nombre. El alta no crea contenidos ni asignaciones. |
| Asignación (`wiki_assignments`) | Clave `(user_id, wiki_id)`; autor y fecha de asignación existentes. | No se añade un tipo de permiso. Usar la semántica de lectura vigente, incluida la actividad del usuario y del agente. Revocar afecta la próxima consulta. |
| Registro de conocimiento (`records`) | `id`, `wiki_id`; revisiones relacionadas. | Contar cada fila una vez en su wiki; editar no aumenta este total. |
| Fuente original (`original_sources`) | `id`, `wiki_id`; puede estar referenciada por varias revisiones. | Contar cada original persistido una vez, con o sin referencias; no deduplicar por hash. |
| Aportación confirmada (`wiki_events`) | `wiki_id`, `actor_id`, `created_at`, `sequence`, `action`. | Derivar actividad de la lista positiva en [research.md](research.md). `sequence` solo desempata internamente; no se expone. |
| Resumen de supervisión | Wiki, agente, dos conteos, última fecha y actividad reciente con fecha y autor condicionado. | Proyectar en el servidor; sin autor/ID de autor cuando no hay asignación de lectura. Nunca incluye cuerpos, referencias de eventos ni secretos. |

## Transiciones e invariantes

1. **Agente elegible → wiki creada:** validar rol, agente y nombre dentro del flujo de escritura; confirmar solo tras el commit. Dos altas concurrentes no pueden crear más de una wiki. Sin candidato o ante fallo, ninguna creación nueva se confirma.
2. **Wiki vacía → wiki con actividad:** solo una aportación confirmada cambia los contadores que correspondan y genera actividad elegible. Editar puede actualizar la última fecha sin aumentar totales.
3. **Proyección sin asignación → con asignación → sin asignación:** los conteos y fechas se mantienen; nombre de autor y acceso a contenido siguen la asignación efectiva para la siguiente solicitud. El rol sigue siendo obligatorio para supervisar.
4. **Carga fallida ≠ wiki vacía:** cero registros, cero fuentes y ninguna actividad confirmada representan vacío; no usar valores por defecto para cubrir una lectura fallida.
