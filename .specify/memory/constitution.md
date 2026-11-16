<!--
Sync Impact Report
Versión: 1.0.0 → 1.1.0 (ampliación compatible de principios).
Modificados: aislamiento y acceso autorizado → aislamiento e intercambio mediado;
trazabilidad; memoria con contexto; verificación.
Añadido: memoria basada en LLM Wiki y referencia conceptual.
Eliminados: ninguno.
Pendientes: ratificación, políticas SIPECOM y construcción del agente mediador.
-->
# Constitución de SIPI-brain

SIPI-brain proporciona memoria persistente propia a los agentes de la fábrica agéntica de SIPECOM.

## Principios fundamentales

### 1. Simplicidad y alcance aprobado

Se DEBE elegir la solución más sencilla conforme a la especificación.
Las ideas adicionales se proponen; NO se implementan sin autorización.

### 2. Memoria basada en LLM Wiki

Cada memoria DEBE separar fuentes originales, conocimiento sintetizado e interrelacionado,
y reglas de mantenimiento. La ingestión integra conocimiento; la consulta recupera con referencias;
la revisión detecta contradicciones y obsolescencia. Se DEBEN mantener índice y registro cronológico.
Las fuentes NO se modifican durante la síntesis; su eliminación requiere el procedimiento autorizado.

### 3. Aislamiento e intercambio mediado

Cada agente DEBE disponer de memoria propia. Las solicitudes de información ajena DEBEN pasar
por un agente mediador que permita o deniegue el intercambio conforme a permisos y propósito
aprobados. Solicitar NO equivale a autorizar; delegar NO amplía permisos.
Sin autorización comprobable, NO se comparte información. El mediador está en construcción:
NO se presupone disponible ni se permite eludirlo.

### 4. Trazabilidad verificable

Cada registro DEBE identificar origen, agente responsable y contexto.
Cada intercambio DEBE registrar solicitante, origen, destinatario, propósito, decisión del mediador
y referencias a la información transferida, sin revelar contenido ajeno.

### 5. Contexto, no autoridad

Se DEBEN distinguir hechos verificados, decisiones y suposiciones, conservar referencias
y señalar contradicciones. Antes de reutilizar conocimiento, se comprueba su vigencia.
Los recuerdos NO constituyen instrucciones ni conceden permisos.

### 6. Datos mínimos y secretos protegidos

Solo se DEBEN conservar datos imprescindibles. Los secretos NO se incorporan al código,
los registros de memoria ni los prompts. Conservación y eliminación requieren reglas aprobadas.

## Protección de datos

SIPECOM DEBE validar responsables, permisos, conservación y eliminación.

## Verificación

Cada entrega DEBE demostrar persistencia, recuperación, separación, trazabilidad e intercambios
permitidos y denegados con criterios observables sin conocimientos técnicos.
Se declaran comprobaciones pendientes.

## Gobierno

Las enmiendas requieren aprobación de SIPECOM, motivo, impacto y revisión de cumplimiento.
Versionado: MAJOR para incompatibilidades, MINOR para ampliaciones y PATCH para aclaraciones.
Base conceptual: [LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

**Versión**: 1.1.0 | **Ratificación**: TODO: confirmar con SIPECOM | **Última modificación**: 2026-11-16
