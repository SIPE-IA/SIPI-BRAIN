# Feature Specification: Alta de agentes y prueba de conexión desde administración

**Feature Branch**: `codex/003-admin-agent-onboarding`

**Created**: 2026-11-18

**Status**: Draft

**Input**: User description: “Sí, agrega al panel de admin la opción de registrar un agente, crearle su wiki y probar la conexión desde la interfaz, sin tener que tocar el backend manualmente. pero esto seria una nueva spec verdad vamos a crear un spec segun la pantilla”

## Clarifications

### Session 2026-11-18

- Q: Para dar por terminada la primera entrega, ¿basta con probar la conexión desde el panel contra un servicio de prueba controlado o debe funcionar con al menos un agente SIPI real? → A: Exigir una conexión verificada desde el panel con al menos un agente SIPI real antes de cerrar la primera entrega.
- Q: ¿El panel debe limitarse a seleccionar destinos ya aprobados o también debe permitir aprobar destinos nuevos? → A: El panel solo permite seleccionar destinos ya aprobados para el entorno; no permite aprobar destinos nuevos.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Registrar un agente desde el panel (Priority: P1)

Como administrador, quiero registrar un agente desde el panel, incluso en una instalación sin agentes, sin ejecutar comandos ni editar archivos o datos manualmente.

**Por qué esta prioridad**: El alta elimina el bloqueo inicial que hoy impide crear una wiki cuando no hay agentes elegibles.

**Prueba independiente**: Desde una instalación vacía con una cuenta administradora, registrar un agente con un identificador y un nombre. Recargar el panel y verificar que sigue disponible para crearle una wiki.

**Escenarios de aceptación**:

1. **Dado** un administrador en una instalación sin agentes, **cuando** registra un identificador nuevo y un nombre válido, **entonces** el panel confirma el alta y el agente aparece como elegible.
2. **Dado** un identificador que ya existe, **cuando** el administrador intenta reutilizarlo, **entonces** el panel informa el duplicado y conserva el agente original.
3. **Dado** un usuario sin rol de administrador, **cuando** intenta realizar el alta, **entonces** se rechaza la operación y no se crea ningún agente.

---

### User Story 2 - Crear la wiki del agente (Priority: P2)

Como administrador, quiero crear la wiki del agente recién registrado desde el mismo panel para completar su alta sin configuración manual.

**Por qué esta prioridad**: Registrar la identidad no le da todavía un espacio de conocimiento al agente.

**Prueba independiente**: Con un agente activo sin wiki, crearla desde el panel y comprobar que aparece asociada a ese agente después de recargar.

**Escenarios de aceptación**:

1. **Dado** un agente activo sin wiki, **cuando** el administrador crea una con un nombre válido, **entonces** el panel muestra la wiki y su agente asociado, sin conceder acceso automáticamente.
2. **Dado** un agente con wiki o un agente inactivo, **cuando** se intenta crear una wiki para él, **entonces** se rechaza la operación y se conserva el estado anterior.
3. **Dado** un agente activo sin conexión verificada, **cuando** el administrador crea su wiki, **entonces** la creación no depende del resultado de una prueba de conexión.

---

### User Story 3 - Configurar y probar la conexión (Priority: P3)

Como administrador, quiero seleccionar un destino ya aprobado para un agente registrado y probar la conexión desde el panel para saber si SIPI-brain puede verificar el servicio esperado sin consultar ni transferir conocimiento.

**Por qué esta prioridad**: Un agente registrado o una wiki creada no demuestran que el servicio esté conectado.

**Prueba independiente**: Con un agente ya registrado, configurar un destino autorizado y probarlo desde el panel con un servicio controlado. Repetir con un servicio no disponible y con una identidad distinta. No se requiere crear una wiki para esta prueba.

**Escenarios de aceptación**:

1. **Dado** un destino autorizado y compatible que responde con la identidad esperada, **cuando** el administrador pulsa **Test connection**, **entonces** el panel confirma que la conexión fue verificada.
2. **Dado** un destino autorizado, **cuando** no responde a tiempo o no se puede comprobar la identidad esperada, **entonces** el panel distingue el motivo y no muestra la conexión como verificada.
3. **Dado** un destino no aprobado para el entorno, **cuando** el administrador configura la conexión, **entonces** el destino no aparece en el selector. Si se intenta guardar o probar de todos modos, el sistema rechaza la operación sin contactar ese destino.
4. **Dada** una prueba fallida, **cuando** el administrador vuelve al panel, **entonces** el agente y su wiki, si existe, se conservan y la prueba no ha cambiado ningún permiso.
5. **Dado** un agente SIPI real con contrato compatible, destino autorizado y un método aprobado para verificar su identidad, **cuando** el administrador ejecuta **Test connection** desde el panel, **entonces** el panel confirma la identidad esperada sin acceder a contenido de wikis ni conceder permisos.

### Edge Cases

- Si dos administradores registran a la vez el mismo identificador, se conserva un solo agente. Si solicitan dos wikis simultáneas para el mismo agente, se conserva una sola wiki.
- Un identificador o nombre vacío no permite completar el alta. Si no hay destinos aprobados y compatibles, el panel indica que no se puede probar la conexión, pero permite registrar el agente y crear su wiki.
- Si una respuesta indica una redirección a un destino no autorizado, no se sigue. Una respuesta accesible sin identidad verificable no cuenta como éxito.
- Si cambia la configuración de conexión, un resultado anterior no se muestra como verificación de la nueva configuración.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: El panel DEBE permitir que un administrador registre un agente activo mediante un identificador único y un nombre visible, sin depender de agentes precargados. El sistema DEBE persistir el alta y rechazar identificadores duplicados sin sobrescribir.
- **FR-002**: El panel DEBE permitir crear una wiki con nombre para un agente activo sin wiki, incluido uno recién registrado. El sistema DEBE aplicar el límite de una wiki por agente incluso ante solicitudes simultáneas.
- **FR-003**: El panel DEBE permitir guardar la configuración de conexión de un agente registrado, incluyendo un destino compatible seleccionado del conjunto de destinos ya aprobados para el entorno y la identidad esperada. El panel NO DEBE admitir destinos escritos libremente ni ofrecer una operación para aprobar destinos nuevos. Registrar un agente o crearle una wiki NO DEBE depender de tener una conexión configurada o verificada.
- **FR-004**: El sistema DEBE limitar el alta, la creación, la configuración y la prueba a administradores, también si la operación se intenta fuera del panel. El rol de administrador y estas acciones NO DEBEN otorgar acceso a la wiki sin una asignación explícita.
- **FR-005**: La prueba DEBE iniciarse explícitamente desde el panel y contactar únicamente destinos que sigan aprobados para el entorno en el momento de la prueba. La revocación se hace efectiva cuando el backend activa la nueva versión del catálogo del entorno, sin reiniciar; editar o publicar el cambio no equivale a activarlo. Si la aprobación ya no está vigente antes de que comience el contacto, la prueba se DEBE rechazar sin contactar el destino, aunque la solicitud ya estuviera en cola. Si el contacto ya comenzó y la revocación se hace efectiva antes de confirmar el resultado, la prueba NO DEBE devolver una verificación exitosa. NO DEBE descubrir servicios automáticamente ni seguir redirecciones a destinos no autorizados.
- **FR-006**: La prueba DEBE finalizar dentro de un plazo acotado y mostrar si la conexión fue verificada, si el servicio no respondió, si no se pudo verificar la identidad o si la configuración no se puede probar. Solo se DEBE confirmar el éxito si responde la identidad esperada del agente para la configuración probada. La primera entrega DEBE admitir y demostrar esta prueba con al menos un agente SIPI real.
- **FR-007**: La prueba NO DEBE consultar, publicar ni intercambiar contenido de wikis, modificar asignaciones ni cambiar el agente o su wiki ante un fallo. Los resultados y registros NO DEBEN exponer secretos.

### Key Entities

- **Agente**: Identidad única con nombre visible y estado; titular de como máximo una wiki.
- **Wiki**: Espacio de conocimiento asociado a un agente. El acceso de usuarios sigue administrándose por asignaciones.
- **Configuración de conexión**: Destino seleccionado entre los ya aprobados para el entorno e identidad esperada para un agente.
- **Resultado de prueba**: Resultado de verificar una configuración concreta; no concede acceso ni demuestra intercambio de conocimiento.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: En una instalación sin agentes, un administrador registra un agente, crea su wiki, configura su destino y ejecuta la prueba desde el panel, con cero comandos y cero ediciones manuales de backend. Tras recargar, el agente, la wiki y su configuración se conservan.
- **SC-002**: Al intentar dos altas simultáneas con el mismo identificador y dos creaciones simultáneas de wiki para ese agente, queda exactamente un agente y una wiki. Un usuario sin rol administrativo no puede completar ninguna de las cuatro operaciones protegidas.
- **SC-003**: En una prueba controlada de cuatro casos —identidad correcta, servicio sin respuesta, identidad incorrecta y destino sin aprobación vigente— solo el primero muestra éxito; el cuarto realiza cero contactos. Ningún caso lee o transfiere conocimiento ni modifica asignaciones.

- **SC-004**: Antes de cerrar la primera entrega, un administrador obtiene desde el panel al menos una verificación exitosa con un agente SIPI real autorizado. Un servicio controlado o una respuesta sin identidad comprobada no satisfacen este criterio.

## Assumptions

- Se conserva el piloto sintético existente. Los servicios controlados permiten probar de forma aislada; no sustituyen la evidencia requerida con un agente SIPI real.
- La aceptación de la primera entrega queda pendiente si no se dispone del contrato compatible, el destino expresamente autorizado y el método aprobado de verificación de identidad para al menos un agente SIPI real. Las credenciales, cuando sean necesarias, se manejan por los mecanismos aprobados del entorno; esta entrega no define una bóveda nueva.
- Fuera de alcance: aprobar o administrar destinos desde el panel, descubrimiento automático, intercambio de conocimiento entre agentes y configuración del puerto del propio SIPI-brain. El intercambio real autorizado sigue en el alcance de la especificación 001.
