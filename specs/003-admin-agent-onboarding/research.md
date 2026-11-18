# Investigación: alta de agentes y prueba de conexión

## 1. Alta y persistencia

**Decisión:** Registrar agentes activos en la tabla existente mediante inserción sin sobrescritura. Reutilizar el alta de wiki de la 002 y sus restricciones SQLite (clave primaria de agente y unicidad de wiki por agente).

**Justificación:** Las restricciones en base de datos dan el mismo resultado bajo concurrencia que en solicitudes secuenciales. Los endpoints existentes de wikis y asignaciones conservan su semántica. El panel no concede acceso por registrar el agente ni por crearle la wiki.

**Alternativas consideradas:** Un upsert modificaría al agente existente ante un duplicado. Una consulta de existencia sin restricción no excluye carreras. Un alta conjunta haría depender una historia de la otra.

**Evidencia local:** [Esquema SQLite](../../backend/migrations/0002_wikis.sql), [servicio](../../backend/src/services.rs), [contrato 002](../002-wiki-admin-monitoring/contracts/wiki-admin.md).

## 2. Catálogo y autorización

**Decisión:** Usar un catálogo autoritativo aprovisionado fuera del panel, vacío por defecto. Publicar solo claves opacas y etiquetas no sensibles de destinos compatibles. Reutilizar sesión, validación de origen, CSRF y control de rol administrativo vigente. Al guardar o probar, resolver nuevamente la clave contra la configuración aprobada del entorno. No seguir redirecciones; exigir que el adaptador mantenga el destino efectivo dentro de la aprobación y falle cerrado si no puede garantizarlo.

**Justificación:** La lista visible no otorga autoridad. La configuración de un agente no puede aprobar un destino ni obligar al backend a ejecutar un contacto arbitrario.

**Alternativas consideradas:** Aceptar una URL del navegador mezcla selección y aprobación. Persistir una copia de la aprobación en la fila del agente permite usar datos desactualizados. Un catálogo administrable en el panel está fuera de alcance.

**Evidencia local:** [Guardas API](../../backend/src/api.rs), [control de acceso](../../backend/src/access.rs).

## 3. Identidad, plazo y vigencia

**Decisión:** Adaptador explícito de solo verificación de identidad; se selecciona por contrato compatible del catálogo, no por datos libres del cliente. Aplicar un límite total inicial de 10 s, con hasta 3 s para establecer conexión cuando el transporte permita ese límite, sin reintentos automáticos. Probar fuera de transacciones de escritura. El resultado es efímero y se correlaciona con la revisión de configuración y la aprobación usada; una edición o cambio de aprobación durante la prueba impide devolver éxito obsoleto.

**Justificación:** Alcanzabilidad, una respuesta HTTP exitosa o un identificador autodeclarado no demuestran identidad. Una revisión que cambia en cada modificación efectiva distingue A → B → A. Un resultado operativo no es un permiso ni un estado de disponibilidad permanente.

**Alternativas consideradas:** Ping/TCP solo mide transporte. Inventar un endpoint o adoptar un transporte antes de comprobar el contrato real oculta la dependencia. Guardar el resultado añadiría estado persistido y políticas de caducidad no requeridas.

**Evidencia local:** El [contrato interno 001](../001-sipibrain-collective-memory/contracts/agent-exchange.md) propone plazos, no un protocolo SIPI. No hay evidencia local de un adaptador real compatible. Protocolo, contrato de identidad, autorización de destino/operación/canal y la evidencia de una prueba exitosa siguen siendo prerrequisitos de la integración real, **no** decisiones para rellenar con supuestos. Un doble comprueba el contrato local, pero no satisface SC-004.
