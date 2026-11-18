# Feature Specification: SIPI-brain — Creación y supervisión de LLMwikis

**Feature Branch**: `codex/002-wiki-admin-monitoring`

**Created**: 2026-11-17

**Status**: Publicado

**Input**: Crear LLMwikis desde la interfaz administrativa y supervisar cómo se llenan, sin ampliar implícitamente el acceso a las memorias.

**Antecedente**: [Especificación 001](../001-sipibrain-collective-memory/spec.md), en particular FR-001 (wiki propia y asignaciones) y FR-006/SC-002 (conexión e intercambio autorizado). Su [registro de tareas](../001-sipibrain-collective-memory/tasks.md) conserva el estado de implementación y verificación; esta especificación no lo modifica ni da por completadas tareas pendientes.

## Clarifications

### Session 2026-11-17

- Q: ¿Qué acceso tiene el administrador a las wikis sin asignación? → A: Puede supervisar los metadatos de todas las wikis, pero necesita una asignación a la wiki correspondiente para abrir su contenido.
- Q: ¿Debe el administrador ver quién hizo cada aportación en una wiki que no tiene asignada? → A: Puede ver las fechas de actividad de todas las wikis, pero solo puede ver el autor y abrir el contenido si tiene una asignación de lectura a esa wiki.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Crear LLMwikis (Priority: P1)

Como administrador, quiero crear desde la interfaz una LLMwiki asociada a un agente para gestionar su memoria sin editar datos manualmente.

**Why this priority**: Permite incorporar la memoria de un agente sin intervención manual sobre los datos.

**Independent Test**: Crear una wiki para un agente sin wiki, reabrir el sistema y gestionarla mediante las asignaciones existentes. No requiere la vista de supervisión.

**Acceptance Scenarios**:

1. **Dado** un administrador autenticado, **cuando** abre el listado administrativo, **entonces** encuentra una opción visible para crear una wiki y puede indicar su nombre y seleccionar el agente al que pertenece.
2. **Dado** un nombre válido y un agente sin wiki, **cuando** se confirma la creación, **entonces** la wiki aparece en el listado administrativo con su agente asociado y sigue allí después de cerrar y reabrir el sistema.
3. **Dado** un agente que ya tiene wiki, **cuando** se intenta crear otra para él, incluso con otro nombre, **entonces** se rechaza la creación y se conserva la wiki existente.
4. **Dado** un fallo de guardado, **cuando** el sistema no puede confirmar la creación, **entonces** muestra un error, no comunica éxito ni lista como creada una wiki no confirmada.
5. **Dado** un usuario sin rol administrador, **cuando** intenta crear una wiki, incluso mediante una solicitud directa fuera de la interfaz, **entonces** se deniega la operación y no se crea la wiki.
6. **Dada** una wiki nueva, **cuando** el administrador le asigna un usuario por el flujo existente, **entonces** ese usuario obtiene el acceso que le corresponde por la asignación y los demás no reciben nuevo acceso automático.

---

### User Story 2 - Supervisar el crecimiento (Priority: P2)

Como administrador, quiero visualizar cómo se van llenando las wikis para distinguir las que están vacías de las que reciben nuevas aportaciones.

**Why this priority**: Permite ver dónde crece la memoria sin necesitar acceso al contenido para realizar la supervisión.

**Independent Test**: Usar dos wikis preexistentes de agentes distintos, una vacía y otra con aportaciones conocidas; agregar una aportación autorizada a la segunda y actualizar la vista. No requiere el flujo nuevo de creación.

**Acceptance Scenarios**:

1. **Dada** una wiki recién creada y sin aportaciones, **cuando** la supervisión carga correctamente, **entonces** muestra su nombre y agente, cero registros, cero fuentes, ninguna fecha ni actividad de aportación y un estado explícito de wiki vacía.
2. **Dadas** dos wikis distintas, **cuando** se guarda una aportación autorizada que agrega un registro y dos fuentes originales a una de ellas y el administrador actualiza la vista, **entonces** sus contadores aumentan por separado en uno y dos y se actualizan la fecha de última aportación y la actividad reciente; el autor solo se muestra si el administrador tiene asignación de lectura a esa wiki. Los datos de la otra wiki no cambian.
3. **Dado** un conjunto conocido de aportaciones, **cuando** se consulta la supervisión, **entonces** los totales, fechas y actividad visible corresponden a los cambios persistidos de esa wiki, no a intentos rechazados ni a cambios de otra.
4. **Dado** un fallo al cargar la supervisión, **cuando** el administrador consulta o actualiza la vista, **entonces** se informa el fallo sin reemplazar los datos desconocidos por ceros ni rotular la wiki como vacía.
5. **Dado** un usuario sin rol administrador, **cuando** intenta abrir la supervisión administrativa, incluso fuera de la interfaz, **entonces** se deniega el acceso a esa vista y sus datos.
6. **Dadas** dos wikis con aportaciones y un administrador con asignación de lectura a solo una de ellas, **cuando** consulta la supervisión e intenta abrir el contenido de ambas, **entonces** ve las fechas de actividad de las dos, pero solo ve los autores y puede abrir el contenido de la wiki asignada. Fuera de la interfaz tampoco se revela la autoría ni se permite leer el contenido de la otra wiki. No se revelan secretos.

### Edge Cases

- Un nombre vacío o un agente inexistente impide confirmar la creación; no se genera una wiki.
- Si dos solicitudes compiten por crear una wiki para el mismo agente, se confirma como máximo una y nunca se crean duplicados.
- Si una misma fuente original está referenciada por varios registros, se cuenta una sola vez dentro de su wiki. Una revisión persistida puede actualizar la fecha y la actividad sin aumentar los contadores.
- Una aportación cuyo autor no se muestra por falta de asignación conserva su fecha visible y no se confunde con una wiki vacía.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El listado administrativo DEBE mostrar una opción para crear una wiki que solicite un nombre no vacío y un agente existente. El sistema DEBE denegar la creación sin rol de administrador, incluso fuera de la interfaz.
- **FR-002**: El sistema DEBE impedir más de una wiki por agente, incluso con nombres distintos o solicitudes concurrentes, sin reemplazar una wiki existente.
- **FR-003**: Una creación confirmada DEBE aparecer en el listado administrativo y persistir tras reabrir el sistema. Un fallo DEBE informarse sin confirmar éxito ni mostrar una creación no confirmada como exitosa.
- **FR-004**: Las wikis nuevas DEBEN poder gestionarse mediante el flujo existente de asignaciones. Crear o asignar una wiki no DEBE conceder a los agentes acceso automático a otras memorias ni eludir la mediación.
- **FR-005**: Solo un administrador DEBE acceder a la supervisión administrativa, que DEBE incluir todas las wikis sin exigir una asignación. Por wiki, la vista DEBE mostrar nombre, agente asociado, totales separados de registros de conocimiento y fuentes originales, fecha de última aportación y fechas de actividad reciente. El sistema DEBE revelar la identidad del autor de una aportación solo si el administrador tiene una asignación de lectura a esa wiki, incluso cuando consulta la supervisión fuera de la interfaz. La supervisión DEBE excluir contenido de aportaciones, valores secretos y referencias a secretos.
- **FR-006**: Al actualizar la vista, las cifras y la actividad DEBEN derivarse de las aportaciones persistidas de la wiki correspondiente. La última fecha corresponde al último cambio de aportación guardado, no a la creación de la wiki, a una asignación ni a un intento fallido. La actividad reciente DEBE aparecer de más reciente a más antigua.
- **FR-007**: La vista DEBE distinguir el estado vacío confirmado de un fallo de carga. Una wiki nueva sin aportaciones no DEBE presentar su creación administrativa como actividad de aportación.
- **FR-008**: Para abrir el contenido de una wiki, el administrador DEBE tener una asignación que autorice la lectura de esa wiki, conforme a la especificación 001. Sin esa asignación, la lectura DEBE denegarse también fuera de la interfaz. El rol administrativo no sustituye la asignación.

### Key Entities *(include if feature involves data)*

- **Agente**: Titular de como máximo una LLMwiki; seleccionable entre los agentes existentes.
- **LLMwiki**: Memoria propia de un agente con nombre visible, registros de conocimiento y fuentes originales diferenciados.
- **Aportación persistida**: Cambio guardado en la memoria, con wiki, autor y fecha; puede agregar un registro o una fuente, o revisar un registro existente.
- **Asignación**: Relación existente que autoriza a un usuario a consultar o aportar en una wiki determinada según sus permisos.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: En una prueba administrativa, la wiki creada aparece una vez en el listado antes y después de reabrir el sistema y se puede asignar a un usuario sin concederle acceso a otras wikis.
- **SC-002**: Los intentos de creación duplicada, sin rol y con guardado fallido producen cero confirmaciones indebidas. Tras dos solicitudes simultáneas para el mismo agente inicialmente sin wiki, queda una sola wiki.
- **SC-003**: Con dos wikis y una aportación persistida de un registro y dos fuentes para una de ellas, la actualización muestra un cambio de +1 y +2 respectivamente solo en esa wiki. La fecha visible y, cuando hay asignación de lectura, el autor coinciden con lo persistido.
- **SC-004**: Los estados de wiki vacía, wiki con aportaciones y fallo de carga se distinguen en los tres casos de prueba. Las comprobaciones de supervisión no autorizada y de secretos muestran cero datos restringidos.
- **SC-005**: En una prueba con dos wikis con aportaciones y una única asignación de lectura, el administrador ve las fechas de actividad de ambas y puede ver los autores y abrir el contenido de una sola. La wiki no asignada revela cero identidades de autores y cero contenido, incluso fuera de la interfaz.

## Assumptions

- La creación selecciona un agente existente; no incluye alta de agentes. Los nombres de las wikis no determinan su pertenencia ni sustituyen al agente como criterio de unicidad.
- El piloto anterior proporciona las asignaciones y el flujo de aportación como antecedentes. Su estado sigue documentado en las tareas de la especificación 001. Esta supervisión no depende del intercambio en vivo entre agentes.
- Quedan fuera la eliminación de wikis, las gráficas y la actualización en tiempo real.
- **Puertos, alcance pendiente**: La especificación 001 (FR-006/SC-002) ya prevé comprobar los puertos configurados de SIPI Estimador e Insight y verificar por separado un intercambio real autorizado. Las direcciones y puertos concretos, los permisos de prueba y la mediación siguen pendientes en ese alcance (T036–T038). Esta especificación no añade configuración de puertos. Si la necesidad fuera configurar desde la interfaz el puerto del propio SIPI-brain, sería una petición distinta que requiere aclaración y autorización de alcance.
