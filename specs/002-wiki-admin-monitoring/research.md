# Fase 0: Decisiones de diseño

## Alta sobre el catálogo existente

- **Decisión:** ofrecer agentes activos sin wiki, validar nuevamente en el servidor dentro de `db::write` y conservar `UNIQUE(wikis.agent_id)` como garantía final. Registrar una wiki con su nombre y agente, sin asignaciones automáticas.
- **Fundamento:** [0002_wikis.sql](../../backend/migrations/0002_wikis.sql) ya tiene las relaciones y restricciones; [db.rs](../../backend/src/db.rs) y [access.rs](../../backend/src/access.rs) ofrecen escritura transaccional y comprobación de rol vigente. [pilot.rs](../../backend/src/pilot.rs) crea juntos sus agentes y wikis de ejemplo: el seed actual no deja ningún candidato. Usar un agente elegible independiente en pruebas; para uso operativo, confirmar que existe un agente elegible provisionado por una vía autorizada. Sin candidatos, mostrar el estado explícito y no inventar un agente.
- **Alternativas descartadas:** crear agentes al crear la wiki amplía el alcance; validar duplicados solo en la interfaz no cubre carreras ni llamadas directas.

## Supervisión separada de la lectura

- **Decisión:** añadir una proyección administrativa de solo lectura, calculada en una lectura coherente. Comprobar rol activo para supervisar y la asignación de lectura vigente por wiki para incluir la identidad de sus autores. Las fechas no dependen de la asignación. Mantener sin cambios los contratos de contenido e historial.
- **Fundamento:** [access.rs](../../backend/src/access.rs) exige usuario y agente activos en las asignaciones efectivas. El historial actual en [services.rs](../../backend/src/services.rs) exige asignación y devuelve actor, acción y referencia: exponerlo en supervisión revelaría más de lo aprobado. La proyección nunca contiene cuerpos, identificadores de eventos o referencias, ni secretos.
- **Alternativas descartadas:** ocultar autores en Vue deja los datos en la respuesta; relajar el guard de asignación rompería el aislamiento. No crear un modelo de permisos nuevo: la asignación vigente del MVP ya habilita lectura.

## Contadores y actividad desde datos confirmados

- **Decisión:** contar por separado filas de `records` y `original_sources` de cada wiki; no contar referencias ni deduplicar por hash. Derivar actividad del historial persistido con lista positiva de eventos: `record_created`, `source_created`, `revision_saved`, `proposal_pending` y `conflict_resolved` cuando se guarda la revisión resultante. Una propuesta pendiente guardada cuenta como actividad, no como publicación de contenido. La última fecha se calcula con todos los eventos elegibles; la lista reciente se limita a los diez más recientes por wiki, con secuencia descendente como desempate cronológico. El límite es una decisión inicial de presentación, no un límite del historial.
- **Fundamento:** [0004_sources.sql](../../backend/migrations/0004_sources.sql) separa las fuentes de sus referencias, y [0005_review_events.sql](../../backend/migrations/0005_review_events.sql) define un historial inmutable. Las escrituras de [services.rs](../../backend/src/services.rs) registran la actividad en la misma transacción. El seed usa el flujo de guardado existente. Excluir asignaciones, apertura de wiki, mantenimiento, revisiones que solo anotan estado de conocimiento, enlaces y reintentos que no guardan cambios. Ordenar por instante UTC, no por texto con fracciones de longitud variable.
- **Alternativas descartadas:** una tabla nueva de métricas exige sincronización y reconciliación; contar eventos en lugar de entidades confunde crecimiento del stock con actividad.

**Cierre:** no quedan decisiones de producto bloqueantes para planificar la 002. La disponibilidad de un agente elegible es un prerrequisito operativo, no autorización para añadir alta de agentes o editar datos manualmente. Los puertos y la mediación real siguen en la 001.
