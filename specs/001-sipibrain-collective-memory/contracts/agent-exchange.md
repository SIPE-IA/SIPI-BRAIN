# Agent Exchange Contract: conexión y recuperación autorizada

Contrato **interno propuesto** de adaptadores del backend Rust para servicios existentes. Vue 3 + TypeScript consume la API local y no accede directamente a servicios, credenciales ni wikis de otros agentes. No afirma que los agentes implementen estas operaciones ni prescribe rutas HTTP ficticias para ellos. Las rutas y esquemas reales deben proporcionarlos sus responsables. SIPI NEXUS no participa.

## Configuración previa obligatoria

Por cada servicio: identidad esperada, destino y puerto explícitos, protocolo, comprobación de identidad/estado según su contrato real, referencia al canal de autenticación autorizado y permiso para ejecutar la prueba. El mediador necesita además contrato y mecanismo de verificación de decisiones. Nada se descubre por escaneo ni se obtiene de sesiones ambientales. Remoto, si se solicita después, requiere autorización de destino, operación y credencial/sesión.

Por defecto no hay destinos configurados ni adaptadores reales habilitados. Para el piloto local se espera configuración de loopback; no se extrapola autorización a redes o equipos distintos. No seguir redirecciones a destinos no autorizados, aceptar direcciones de red del cuerpo del registro ni convertir una consulta de conocimiento en una solicitud de red arbitraria.

## Operaciones internas a adaptar

| Operación | Entrada | Salida propuesta |
|---|---|---|
| `check_connection` | Identidad y configuración de un servicio | reachable/unreachable, comprobación de identidad del servicio y error no sensible; nunca un veredicto de intercambio |
| `authorize_exchange` | Identidad autenticada del solicitante, wiki origen, destinatario, propósito, proyecto/cliente y referencias | Decisión verificable permitida/denegada, ámbito exacto y vencimiento; o unavailable |
| `retrieve_authorized` | Misma solicitud y decisión válida comprobada por el servidor | Solo RFDs/referencias autorizados con contenido, proyecto, procedencia y revisión; o fallo sin contenido ajeno |
| `record_receipt` | Identidades, propósito, decisión, referencias y resultado | Recibo trazable que no contiene secretos ni cuerpos completos |

Los nombres anteriores son puntos de diseño internos, no pruebas de existencia de endpoints. Los RFDs son requerimientos funcionales individuales asociados a sus proyectos. Un adaptador no debe generarlos ni convertir ausencia de datos en una respuesta sintetizada.

## Secuencia de recuperación

1. Validar destino configurado e identidad del solicitante mediante el canal real aprobado. El cuerpo `requester=estimador` no autentica al agente.
2. Comprobar separadamente conexión a Insight y Estimador usando sus contratos. Un socket abierto solo prueba accesibilidad de transporte.
3. Obtener y verificar la decisión del mediador. Vincularla a solicitante, origen, destinatario, propósito, referencias y cliente/proyecto exactos. Una decisión caducada, alterada o para otro ámbito no permite contenido.
4. Solo después, leer y entregar la información autorizada. Validar que los RFDs devueltos pertenecen al proyecto/cliente solicitados; no reenviar contenido sobrante recibido de un servicio.
5. Entregar contenido y procedencia a Estimador para la conversación nueva y registrar referencias transferidas. No fusionar las wikis. Si se necesita persistir una copia, registrarla como fuente local con origen y recibo de intercambio, no como acceso libre a la wiki original.
6. Si falla la autorización o no existe el mediador, detenerse sin información ajena. Si se desconoce si una transferencia ya ocurrió, registrar resultado indeterminado sin afirmar entrega o éxito; no repetir ciegamente efectos externos.

Las decisiones permiten un intercambio, no operaciones administrativas ni acceso al almacén de secretos. No hay dependencias de SIPI NEXUS ni lecturas directas alternativas para esquivar al mediador. No mantener transacciones SQLite abiertas durante llamadas de red.

## Límites técnicos propuestos

Espera de conexión acotada a 3 segundos y plazo total de solicitud de 10 segundos como valores iniciales ajustables del adaptador, no SLOs aprobados del negocio. Sin reintentos automáticos de operaciones con efectos externos de resultado desconocido. Cancelación o error genera estado no disponible/indeterminado, nunca datos inventados. Los servicios reales deben declarar límites y semántica antes de elegir reintentos seguros.

## Pruebas y aceptación

- Dobles locales autorizados: identidades válidas e inválidas, permiso/denegación, mediador ausente, decisión caducada, ámbito equivocado, RFD de otro proyecto, destino no configurado, timeout, fallo después de entrega y secreto señuelo ausente de trazas/contexto.
- Prueba negativa de contenido: con mediación ausente o denegada, cero RFDs ajenos, aunque Insight responda en un puerto accesible.
- Prueba real SC-002: ambos servicios configurados responden con identidad correcta; Estimador recupera desde Insight RFD-001/002/003 del Proyecto Alfa y su nota propia, no RFD-004 de Beta, desde otra conversación y sin pedir antecedentes ya guardados.
- Un resultado de dobles puede ser conforme al contrato interno, pero **no aprueba SC-002**. Sin contratos reales, autorización, servicios y mediación disponibles, el resultado de SC-002 es **bloqueado/no ejecutado**, no PASS.

La [matriz de dependencias](../research.md) identifica responsables, entradas y evidencia pendiente. Este diseño no autoriza a inspeccionar máquinas o servicios para completar esos datos.
