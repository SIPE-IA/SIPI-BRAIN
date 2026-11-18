# Feature Specification: SIPI-brain — LLMwikis por agente con aportación colectiva

**Feature Branch**: No creada: no hay hooks de Git configurados y el estado de Git no se pudo verificar.

**Created**: 2026-11-16

**Status**: Publicado — cinco aclaraciones integradas; se mantienen las dependencias y decisiones diferidas del piloto. No constituye aprobación para usar datos o secretos reales.

**Input**: Texto adjunto «1. Objetivo y contexto de negocio SIPI-brain será la memoria colectiva de los t…» y correcciones explícitas de la conversación. Fuente: `C:/Users/kfernandez/.codex/attachments/90019e8f-76f5-4288-9ea6-8188c50eb893/Texto pegado.txt`.

SIPI-brain permitirá que varios programadores de SIPECOM aporten prompts, decisiones e información de trabajo, conservando una LLMwiki propia para cada agente. «Colectiva» describe la participación de varios trabajadores, no una memoria única ni acceso automático entre agentes. El piloto se ejecutará en un dispositivo local. SIPI NEXUS es un sistema totalmente independiente y queda fuera de esta especificación, sin integración presupuesta. Los valores secretos permanecerán separados de las LLMwikis y no se entregarán a agentes ni a servicios externos.

## Clarifications

### Session 2026-11-16

- Aclaración adicional del usuario: «RFD es un requerimento funcional de todos los proyectos». RFD significa requerimiento funcional; se utiliza en todos los proyectos y cada requerimiento se asocia al proyecto que corresponda, sin asumir que uno mismo aplique a todos.

- Q: ¿Qué programadores podrán consultar y aportar información a la LLMwiki de cada agente durante el piloto? → A: B. Cada programador puede consultar y aportar únicamente a las wikis que tenga asignadas. Los secretos quedan fuera de esta decisión.
- Q: ¿La prueba del piloto debe comprobar únicamente la conexión por los puertos de ambos agentes o también el intercambio real de información? → A: B. Comprobar los puertos de SIPI Estimador y SIPI Insight y demostrar que Estimador recupera un RFD de Insight mediante el intercambio autorizado. La comprobación de puertos por sí sola no acredita la interacción.
- Q: ¿Quién podrá asignar a cada programador las wikis que puede consultar y modificar durante el piloto? → A: A. Un administrador del piloto gestiona las asignaciones de todas las wikis. Esta autoridad no concede por sí sola acceso a secretos ni permite eludir la mediación entre agentes.
- Q: ¿Qué debe incluir «recuperar información» de las LLMwikis durante el piloto? → A: A. Buscar, abrir y consultar información guardada. No incluye restaurar documentos borrados ni versiones anteriores; el respaldo de secretos sigue pendiente.
- Q: ¿Qué debe ocurrir si dos programadores modifican el mismo registro al mismo tiempo? → A: B, dejando al administrador a cargo de resolver el caso. Se conservan ambas propuestas por separado para revisión manual, sin elegir automáticamente una como vigente.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Aportar conocimiento entre varios programadores (Priority: P1)

**Origen: HU2, corregida por el usuario.** Como programador de SIPECOM, quiero aportar y consultar prompts, decisiones e información en la LLMwiki del agente correspondiente junto con otros programadores autorizados, para aprovechar el trabajo del equipo y continuar las tareas sin empezar desde cero.

**Why this priority**: Permite que el conocimiento de cada agente se construya con el trabajo de varios programadores, sin mezclar las memorias de agentes diferentes.

**Independent Test**: Dos programadores autorizados guardan aportaciones distintas en la LLMwiki de un mismo agente. Tras reabrir el sistema, ambos recuperan las dos aportaciones con su autoría; la wiki de otro agente permanece separada. No requiere generación de RFDs ni estimaciones.

**Acceptance Scenarios**:

1. **Given** la LLMwiki vacía de un agente, **When** un programador autorizado guarda una decisión y vuelve a abrir SIPI-brain, **Then** recupera el mismo contenido, su procedencia y sus asociaciones en esa wiki.
2. **Given** dos programadores autorizados para la LLMwiki de SIPI Estimador, **When** cada uno guarda un prompt diferente, **Then** ambos pueden recuperar las dos aportaciones y distinguir quién aportó cada una.
3. **Given** contenido asociado a una LLMwiki, cliente y proyecto, **When** se consulta por esas categorías, **Then** se conserva la wiki de pertenencia y la procedencia no cambia.
4. **Given** las LLMwikis separadas de SIPI Estimador y SIPI Insight sin intercambio autorizado, **When** se consulta la wiki de Estimador, **Then** no se incorporan automáticamente aportaciones exclusivas de la wiki de Insight.
5. **Given** un programador con la wiki de Estimador asignada pero no la de Insight, **When** consulta o intenta aportar a Insight, **Then** ambas operaciones se deniegan sin revelar contenido ni cambiar los registros; las mismas operaciones sobre Estimador están permitidas conforme a su asignación.
6. **Given** un administrador del piloto, **When** asigna a un programador la wiki de un agente, **Then** el programador puede consultarla y aportar según esa asignación. Si un programador sin rol de administrador intenta cambiar asignaciones, la operación se deniega y los permisos permanecen sin cambios.
7. **Given** dos programadores autorizados modificando el mismo estado inicial de un registro, **When** guardan propuestas diferentes, **Then** se conservan ambas por separado con autoría, wiki y registro afectados, y estado pendiente de resolución; ninguna se elige automáticamente como vigente y ambas siguen disponibles tras reabrir el sistema.
8. **Given** ese conflicto pendiente, **When** el administrador lo revisa y registra su resolución manual, **Then** se identifica el contenido vigente y se conserva la trazabilidad de ambas propuestas y de la decisión del administrador, sin fusión automática.
9. **Given** ese conflicto pendiente, **When** un programador sin rol de administrador intenta resolverlo, **Then** la operación se deniega y las propuestas y su estado permanecen sin cambios.

---

### User Story 2 - Reutilizar antecedentes de clientes (Priority: P1)

**Origen: HU1.** Como SIPI Estimador, quiero recuperar la información del cliente y los RFDs generados por SIPI Insight, para preparar una estimación con los antecedentes necesarios.

**Why this priority**: Demuestra una recuperación entre agentes con un resultado de negocio concreto, sujeta a la mediación y autorización constitucionales, sin atribuir a SIPI-brain la generación de documentos ni equiparar al mediador con SIPI NEXUS.

**Independent Test**: Con las direcciones y puertos de SIPI Insight y SIPI Estimador expresamente configurados para la prueba, comprobar que ambos servicios responden y realizar una recuperación real y autorizada de RFD-001 desde Insight hacia Estimador. Verificar contenido, cliente y procedencia sin fusionar las wikis. Un puerto accesible no basta para aprobar. Si el mediador no está disponible, esta prueba queda pendiente y no se elude la mediación.

**Acceptance Scenarios**:

1. **Given** tres requerimientos funcionales RFD-001, RFD-002 y RFD-003 del Proyecto Alfa de Cliente Alfa en la LLMwiki de SIPI Insight, una nota en la de SIPI Estimador y un intercambio autorizado por el mediador, **When** SIPI Estimador solicita esos antecedentes, **Then** obtiene los tres requerimientos y la nota con su procedencia y proyecto, sin fusionar las wikis.
2. **Given** además el requerimiento RFD-004 del Proyecto Beta de Cliente Beta, **When** se solicitan antecedentes de Alfa, **Then** RFD-004 no se incorpora a ese resultado ni se atribuye al Proyecto Alfa o a su cliente.
3. **Given** un proyecto sin RFD guardado, **When** se solicita un requerimiento funcional de ese proyecto, **Then** se informa de su ausencia sin generar contenido y presentarlo como un RFD recuperado.
4. **Given** un RFD en otra LLMwiki y una solicitud denegada o sin mediador disponible, **When** SIPI Estimador intenta recuperarlo, **Then** no recibe contenido ajeno y se informa de que el intercambio no está autorizado o no está disponible.
5. **Given** los puertos configurados de ambos servicios accesibles pero una recuperación de RFD fallida, **When** se comprueba su interacción, **Then** la conexión y el intercambio se informan por separado y la prueba de interacción no se da por superada.

---

### User Story 3 - Consultar información manualmente (Priority: P1)

**Origen: HU3.** Como programador de SIPECOM, quiero buscar, consultar y recuperar manualmente la información de las LLMwikis para las que tengo acceso, para comprender los antecedentes de una tarea y utilizarlos en mi trabajo.

**Why this priority**: El conocimiento debe ser utilizable por personas, no solamente por agentes.

**Independent Test**: Con acceso aprobado a la wiki de Insight y un registro previamente guardado, un programador lo encuentra por contenido, agente y cliente, y abre el mismo documento sin recurrir a un agente.

**Acceptance Scenarios**:

1. **Given** RFD-001 guardado para Cliente Alfa por SIPI Insight, **When** el programador busca por Cliente Alfa, por SIPI Insight o por una frase del requerimiento, **Then** encuentra el registro y puede abrir su contenido e identificar origen y cliente.
2. **Given** una búsqueda sin coincidencias, **When** el programador la ejecuta, **Then** ve un resultado vacío explícito, sin documentos inventados.

---

### User Story 4 - Continuar entre conversaciones (Priority: P1)

**Origen: HU4 vigente.** Como agente de SIPECOM, quiero recuperar información guardada en SIPI-brain al iniciar otra conversación, para continuar una tarea sin pedir nuevamente los antecedentes disponibles.

**Why this priority**: Evita que el conocimiento dependa de mantener abierta una conversación particular.

**Independent Test**: Guardar los antecedentes en una conversación, finalizarla e iniciar otra sin copiar su historial; solicitar la misma tarea y recuperar la información desde SIPI-brain.

**Acceptance Scenarios**:

1. **Given** RFD-001, RFD-002, RFD-003 y la nota de Alfa guardados en una conversación anterior, **When** SIPI Estimador consulta esos antecedentes desde una conversación nueva, **Then** recupera los tres requerimientos y la nota sin pedir al usuario que vuelva a introducirlos.

---

### User Story 5 - Crear secretos locales separados (Priority: P1)

**Origen: HU5.** Como programador de SIPECOM, quiero crear secretos desde la interfaz y conservarlos exclusivamente en el dispositivo, para gestionar información sensible sin compartirla con los agentes ni enviarla a servicios externos.

**Why this priority**: La separación de secretos es una restricción explícita del alcance; no puede sustituirse por una promesa de seguridad basada únicamente en que el piloto sea local.

**Independent Test**: Crear ALFA_API_KEY con un valor ficticio único y comprobar confirmación, persistencia local y ausencia del valor en búsquedas generales, contenido compartido, contexto de agentes y comunicaciones externas durante ese flujo.

**Acceptance Scenarios**:

1. **Given** la sección de secretos, **When** el programador registra un nombre y un valor ficticio, **Then** recibe confirmación sin que se vuelva a mostrar el valor completo y el secreto sigue registrado tras reabrir el sistema.
2. **Given** el secreto creado y RFD-001, **When** personas y agentes realizan búsquedas generales y recuperan antecedentes de Alfa, **Then** el valor secreto no aparece en sus resultados ni en el contexto entregado a los agentes.
3. **Given** ese valor ficticio registrado, **When** se crea el secreto y se consulta la memoria, **Then** el valor no sale del dispositivo en comunicaciones externas, incluso si existen comunicaciones de agentes para otros fines.

### Edge Cases

- **Memoria vacía**: mostrar que no hay información y permitir el primer registro.
- **RFD inexistente**: indicar que el requerimiento funcional no está disponible; no inventar un requerimiento y presentarlo como recuperado.
- **Clientes homónimos**: supuesto de borrador: distinguirlos mediante una referencia inequívoca y no utilizar solo el nombre para asociar sus antecedentes.
- **Secreto pegado en una nota normal**: prevención y detección pendientes por decisión del usuario. No se presupone un filtro automático ni se afirma que este caso esté protegido. Usar únicamente datos ficticios hasta definir esa protección.
- **Pérdida o daño del dispositivo**: respaldo y restauración de secretos pendientes; no se promete recuperación ni se presupone copia remota.
- **Ediciones concurrentes del mismo registro**: conservar ambas propuestas por separado y señalarlas como pendientes; no elegir automáticamente una como vigente. El administrador resuelve el caso manualmente. Consultar un conflicto pendiente debe mostrar esa condición y distinguir ambas propuestas, no presentar una como vigente por orden de llegada. Esta detección de colisión de guardados no incluye análisis semántico automático de contradicciones.
- **Participante sin autorización definida**: no habilitar acceso real a información de clientes hasta aprobar el modelo de permisos. Las pruebas de intercambio utilizarán participantes y datos ficticios expresamente habilitados para la prueba.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001 (RF1)**: SIPI-brain DEBE conservar una LLMwiki propia por agente y permitir aportaciones de varios programadores autorizados a una misma wiki. La colaboración humana no DEBE fusionar las memorias ni conceder acceso automático entre agentes. Cada programador DEBE poder consultar y aportar únicamente a las wikis que tenga asignadas; para una wiki no asignada se DEBE denegar tanto la consulta como la aportación, sin revelar contenido ni alterar registros. Esta asignación no concede acceso a secretos. Solo un administrador del piloto DEBE poder gestionar las asignaciones de todas las wikis; un programador sin ese rol no DEBE poder cambiar asignaciones ni concederse acceso.
- **FR-002 (RF2)**: Trabajadores y agentes autorizados DEBEN poder guardar prompts, decisiones e información para tareas posteriores en la LLMwiki correspondiente, conservando autoría, procedencia y asociaciones de agente, cliente y tarea o proyecto.
- **FR-003 (RF3)**: La información guardada DEBE conservar contenido y asociaciones después de cerrar y reabrir SIPI-brain en el dispositivo del piloto.
- **FR-004 (RF4)**: La información DEBE poder organizarse y consultarse por LLMwiki de agente, cliente y tarea o proyecto, sin mezclar las wikis ni tratar la participación de varios programadores como autorización de intercambio entre agentes.
- **FR-005 (RF5)**: SIPI Insight DEBE poder guardar requerimientos funcionales (RFDs) de los distintos proyectos de SIPECOM, asociando cada uno inequívocamente a su proyecto y al cliente cuando corresponda, y conservando a Insight como origen. El concepto no se limita a los proyectos del ejemplo de prueba ni implica que un requerimiento sea común a todos los proyectos.
- **FR-006 (RF6)**: SIPI Estimador DEBE poder recuperar los antecedentes y RFDs del cliente solicitado, con mediación autorizada cuando procedan de otra wiki, sin incluir como propios documentos exclusivos de otro cliente. El piloto DEBE permitir comprobar la conexión mediante los puertos configurados de SIPI Estimador y SIPI Insight y demostrar una recuperación real de un RFD de Insight por Estimador. DEBE distinguir conexión disponible de intercambio completado; un puerto accesible no acredita una recuperación. Si la mediación requerida no está disponible o deniega la solicitud, no DEBE acceder a información ajena; debe informar de la imposibilidad de recuperación sin revelar ese contenido.
- **FR-007 (RF7)**: Los programadores DEBEN poder buscar manualmente por contenido, agente o cliente desde la interfaz.
- **FR-008 (RF8)**: Cada resultado DEBE permitir abrir el contenido e identificar su procedencia y el cliente asociado cuando corresponda.
- **FR-009 (RF9)**: Un agente DEBE poder buscar, abrir y consultar antecedentes guardados desde una conversación distinta de aquella donde se registraron, sin exigir que el usuario vuelva a introducirlos. Esta recuperación no incluye restaurar documentos borrados ni versiones anteriores durante el piloto.
- **FR-010 (RF10)**: La interfaz DEBE permitir crear un secreto con nombre y valor, confirmar su creación sin volver a mostrar el valor completo y conservarlo localmente tras reabrir el sistema.
- **FR-011 (RF11)**: Los valores registrados mediante la sección de secretos DEBEN permanecer en el dispositivo y quedar excluidos de la memoria compartida, búsquedas generales, contexto de agentes y comunicaciones externas. La prevención de su introducción por otras vías sigue pendiente y no se declara resuelta por este requisito.
- **FR-012 (RF12)**: Una consulta sin coincidencias DEBE indicar que no hay resultados, sin presentar contenido generado como información recuperada.
- **FR-013**: Si dos programadores autorizados modifican concurrentemente el mismo registro, SIPI-brain DEBE conservar ambas propuestas separadas, con autoría, wiki, registro afectado y estado pendiente. No DEBE sobrescribir una con otra ni elegir automáticamente una como vigente. Solo un administrador del piloto DEBE poder resolver el caso manualmente e identificar el contenido vigente, conservando las propuestas y la trazabilidad de su decisión. Hasta entonces, la consulta DEBE mostrar que el conflicto está pendiente sin atribuir vigencia automática a ninguna propuesta. Un intento de resolución por un no administrador DEBE denegarse sin alterar propuestas ni estado.

### Key Entities *(include if feature involves data)*

- **Participante**: trabajador o agente de SIPECOM que aporta o consulta conocimiento. Cada programador accede solo a sus wikis asignadas; un administrador gestiona esas asignaciones y resuelve conflictos de edición. Los intercambios entre agentes requieren autorización mediada. Varios programadores pueden contribuir a una misma wiki sin fusionar las memorias.
- **LLMwiki de agente**: memoria propia de un agente, diferenciada de las de otros agentes; reúne aportaciones autorizadas de varios programadores y del agente correspondiente.
- **Registro de conocimiento**: prompt, decisión, antecedente, nota o documento perteneciente a una LLMwiki; conserva contenido, autoría, origen y asociaciones pertinentes.
- **Conflicto de edición y propuestas**: dos modificaciones concurrentes del mismo registro, conservadas por separado con su autoría y estado pendiente; la resolución manual del administrador determina el contenido vigente y conserva la trazabilidad del caso. No equivale a un servicio general de restauración de versiones anteriores.
- **Cliente**: persona u organización cuyos antecedentes se distinguen de los de otros clientes, incluidos los homónimos.
- **Tarea o proyecto**: contexto de trabajo que agrupa registros relacionados dentro de sus respectivas LLMwikis, sin fusionarlas.
- **RFD — Requerimiento funcional**: comportamiento requerido en un proyecto de SIPECOM, generado por SIPI Insight y recuperable por SIPI Estimador y programadores autorizados. El término se utiliza en todos los proyectos; cada RFD conserva su proyecto, procedencia y cliente cuando corresponda. No se define como un documento que agrupa varios requerimientos ni se impone un formato de archivo o plantilla no solicitado.
- **Secreto local**: nombre y valor sensible registrados en la sección independiente del dispositivo; el valor no constituye conocimiento compartido.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: El 100 % de los registros del conjunto de prueba conserva contenido, origen y asociaciones después de cerrar y reabrir SIPI-brain.
- **SC-002**: En la prueba del piloto responden ambos servicios en sus puertos configurados y, desde una conversación nueva, SIPI Estimador recupera realmente los tres requerimientos funcionales RFD-001, RFD-002 y RFD-003 del Proyecto Alfa mediante intercambio autorizado desde Insight y consulta su nota de Alfa en su propia wiki, sin solicitar que se vuelvan a introducir y sin incluir RFD-004 de Beta. Deben cumplirse tanto la conexión como la recuperación; si falla cualquiera, la prueba no se aprueba.
- **SC-003**: Un programador encuentra y abre RFD-001 mediante cada una de las tres búsquedas de prueba: cliente, agente de origen y frase contenida; en todas identifica correctamente su procedencia y cliente.
- **SC-004**: En el 100 % de las consultas de prueba sin coincidencias, el participante recibe una indicación de ausencia, sin antecedentes inventados.
- **SC-005**: El secreto ficticio creado sigue registrado tras reabrir el sistema; se observan cero apariciones de su valor completo en la confirmación posterior al ingreso, búsquedas generales, memoria compartida, contexto de agentes y comunicaciones externas de los flujos probados. Se requiere inspección técnica además de la comprobación visual; no prueba prevención universal de filtraciones.
- **SC-006**: Dos programadores autorizados aportan un prompt cada uno a la misma LLMwiki y ambos recuperan las dos aportaciones con autoría correcta tras reabrir el sistema. La consulta de esa wiki devuelve cero aportaciones exclusivas de otra wiki sin intercambio autorizado. Para una wiki no asignada, las pruebas de consulta y aportación deben ser denegadas sin revelar contenido ni alterar registros. La comprobación con datos reales sigue bloqueada hasta concretar asignaciones y protección de datos.
- **SC-007**: En la prueba de dos modificaciones concurrentes del mismo registro se conservan exactamente las dos propuestas y su autoría tras reabrir el sistema; se muestran como pendientes sin elegir automáticamente una vigente. Un no administrador no puede resolverlas ni cambiar su estado. Tras una resolución manual del administrador, se identifica el contenido vigente y se conservan ambas propuestas y la decisión registrada.

**Objetivo de tiempo propuesto, no aprobado**: recuperar los antecedentes para retomar la tarea de Alfa en menos de un minuto. No es un umbral de aceptación definitivo hasta acordar el dispositivo y el volumen de información, tal como indica el texto de entrada.

## Assumptions

### Alcance confirmado

- El piloto se ejecutará en un dispositivo local; no se deriva de ello que todas las comunicaciones de agentes sean locales ni que se permitan transmitir secretos.
- «Memoria colectiva» significa colaboración de varios programadores en prompts y conocimiento. Cada agente conserva su propia LLMwiki; no significa una memoria única ni consulta libre entre agentes.
- SIPI NEXUS queda totalmente aparte, sin integración ni dependencia presupuesta. No se le asignan tareas en esta especificación ni se trasladan sus funciones a SIPI-brain.
- SIPI Insight genera RFDs y SIPI Estimador prepara estimaciones. SIPI-brain conserva y permite recuperar esos antecedentes; no sustituye a esos agentes.
- Recuperar significa buscar, abrir y consultar información guardada. Por decisión explícita del usuario, la restauración de documentos borrados y versiones anteriores queda fuera del piloto.
- Los identificadores RFD-001, RFD-002 y RFD-003 representan tres requerimientos funcionales del Proyecto Alfa; RFD-004 corresponde al Proyecto Beta. Los nombres de proyectos, clientes, requerimientos y ALFA_API_KEY son ejemplos de prueba, no formatos obligatorios de numeración ni una estructura documental exigida.
- Las pruebas usarán exclusivamente datos y secretos ficticios mientras no se definan permisos y protección de entradas sensibles.

### Fuera de alcance

- SIPI NEXUS y cualquier integración con él.
- Restaurar documentos borrados o versiones anteriores de las LLMwikis durante el piloto. Conservar las propuestas de una edición concurrente para resolverla manualmente sí está incluido y no implica ofrecer restauración general. El respaldo de secretos permanece pendiente y no se decide aquí.
- Filtrado automático de contexto, análisis automático o resolución automática de contradicciones, limpieza y eliminación autónoma de archivos. Esto no deroga la revisión y el mantenimiento requeridos por la constitución.
- Envío de valores secretos a agentes o servicios externos.
- Como exclusiones propuestas del borrador: entrenamiento de modelos, sincronización de secretos entre dispositivos, ejecución automática de operaciones con secretos y generación propia de RFDs o estimaciones.
- Garantías de memoria perfecta o corrección universal de respuestas.

### Dependencias, gobierno y decisiones pendientes

- **Aclaración del usuario**: «Memoria colectiva significa que todos los prompts o cosas no solo vana ser implementados por un programador sino por muchos eso queria decir con memoria colectiva pero de ahi cada agente tiene su propio LLMwiki». Esta aclaración sustituye la interpretación anterior de una memoria única compartida libremente entre agentes; no autoriza cambiar la constitución.
- **Gobierno constitucional conservado**: la memoria propia por agente es compatible con la aclaración. Los principios 3 y 4 de `.specify/memory/constitution.md` mantienen la mediación de intercambios y su trazabilidad. Un mediador no se identifica automáticamente con SIPI NEXUS; este último sigue fuera de alcance. Sin mediador disponible y autorización comprobable no se ejecutan intercambios entre wikis. La revisión y el mantenimiento exigidos por los principios 2 y 5 siguen vigentes; la exclusión de automatización no los deroga. No se modificó la constitución ni se declara disponibilidad de esos mecanismos.
- **PA1 — Significado de RFD, resuelto**: RFD significa requerimiento funcional y se utiliza en todos los proyectos. Cada requerimiento pertenece al proyecto correspondiente. No se presupone una plantilla o formato documental obligatorio; sus detalles de representación podrán concretarse en planificación sin cambiar este significado.
- **PA2 — Permisos, resuelto para el piloto**: cada programador consulta y aporta únicamente a las wikis que tenga asignadas; solo un administrador del piloto gestiona las asignaciones. El piloto incluye la comprobación por puertos de Insight y Estimador y la recuperación autorizada de un RFD de Insight por Estimador. La asignación humana no concede acceso a secretos ni autoriza por sí sola el intercambio entre agentes. La identificación de participantes y la provisión inicial del administrador se concretarán en planificación, sin ampliar estos permisos.
- **PA3 — Recuperación, resuelto para el piloto**: recuperar información significa buscar, abrir y consultar información guardada. Restaurar documentos borrados o versiones anteriores queda fuera del piloto. Esta decisión no define respaldos ni restauración de secretos.
- **PA4 — Gestión de secretos**: falta definir quién puede revelar, modificar y eliminar valores y sus usos locales. Esta especificación solo incluye su creación y separación, no esas operaciones.
- **PA5 — Diferido por el usuario**: el respaldo de secretos y las medidas para evitar que sus valores se introduzcan accidentalmente en la memoria colectiva están por definirse. El piloto local no resuelve estos puntos ni autoriza asumir una solución.
- **PA6 — Vigencia en ediciones concurrentes, resuelto**: se conservan las dos propuestas y el administrador resuelve manualmente cuál es el contenido vigente. No hay selección ni fusión automática. Esto no incorpora detección semántica de contradicciones entre documentos independientes ni restauración general de versiones.
- **Trazabilidad**: se conservan origen, agente responsable cuando corresponda y contexto del conocimiento, sin trasladar secretos a registros o prompts.
- **Métrica temporal, diferida**: el minuto propuesto depende de aprobar entorno y volumen; no se transforma una propuesta en requisito confirmado. Se mantiene pendiente tras alcanzar el límite de cinco aclaraciones.
- **Conexión de servicios**: deben suministrarse las direcciones y puertos concretos, la disponibilidad de los servicios y la autorización para probarlos antes de ejecutar la prueba. No se inventan valores ni se inspeccionan puertos para descubrirlos. El protocolo y la integración concreta se definirán en planificación; SIPI NEXUS sigue fuera de alcance.
