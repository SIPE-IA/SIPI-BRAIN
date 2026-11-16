# Implementation Plan: SIPI-brain — LLMwikis por agente

**Branch**: `001-sipibrain-collective-memory` — identificador lógico de Spec Kit; no se creó una rama porque no existe Git local verificado. **Date**: 2026-11-16. **Spec**: [spec.md](spec.md).

**Input**: `specs/001-sipibrain-collective-memory/spec.md`, incluidas cinco aclaraciones y la definición posterior de RFD.

**Status**: Phase 0 y Phase 1 alineadas con el AGENTS.md actual: Rust + Vue 3 + TypeScript. Diseño del piloto local, sin implementación, builds ni conexiones verificados. No autoriza información ni secretos reales.

## Summary

Stack obligatorio del [AGENTS.md](../../AGENTS.md): **backend Rust; frontend Vue 3 + TypeScript**. Este plan sustituye la propuesta tecnológica anterior sin modificar las reglas de producto. Backend Axum/Tokio con API JSON y cliente Vue/Vite, desplegados bajo un mismo origen local; proxy explícito para desarrollo, sin CORS general.

Cada agente mantiene una LLMwiki lógica propia. Varios programadores aportan solo a las wikis asignadas por el administrador. SQLite conserva fuentes originales inmutables, revisiones Markdown, referencias, revisión humana, índice y eventos. Ante ediciones concurrentes conserva ambas propuestas y el administrador resuelve manualmente, sin ganador automático ni restauración general.

Windows Credential Manager se integra desde Rust mediante windows-sys con persistencia explícita CRED_PERSIST_LOCAL_MACHINE. Valores secretos fuera de base de conocimiento, sesiones, logs y contexto de agentes. Adaptadores de Insight/Estimador solo tras recibir contratos y autorización reales; conexión por puertos no sustituye recuperación de RFDs. SIPI NEXUS sigue fuera del diseño. Un RFD es un requerimiento funcional individual de su proyecto.

## Technical Context

**Language/Version**: Rust para backend; Vue 3 + TypeScript para frontend. Rust/Cargo y Node/npm se fijarán conforme al MSRV/requisitos de los manifiestos elegidos y una compilación real; no se inventan mínimos ni se afirma instalación verificada. Código/UI en inglés; documentación técnica en español profesional.

**Primary Dependencies**: Axum 0.8, Tokio 1, serde 1, SQLx 0.8.6 SQLite; tower-sessions 0.14.0 y tower-sessions-sqlx-store 0.15.0 como línea compatible documentada; RustCrypto argon2 0.6 y windows-sys 0.61.2. Frontend Vue 3/TypeScript/Vite, comprobación vue-tsc y pruebas Vitest. tower-http para servir el bundle. Resolver parches, features y lockfiles al implementar, sin mezclar familias incompatibles. [Investigación](research.md).

**Storage**: SQLite local mediante SQLx, en archivo privado con wiki_id obligatorio y transacciones cortas BEGIN IMMEDIATE sobre una conexión retenida. Sesiones persistentes en SQLite; valores secretos solo en la bóveda Windows, reservas/metadatos aparte. Sin almacenamiento de red o sincronización de secretos.

**Testing**: cargo test para unidad, integración y contrato del backend; SQLite temporal en archivo con conexiones independientes para concurrencia. Vitest para interacción/contrato Vue, vue-tsc para tipos, Vite para build. Bóveda Windows, navegador/API y agentes reales requieren comprobaciones explícitas separadas; mocks no aprueban SC-002.

**Target Platform**: un dispositivo Windows; aplicación local ligada a loopback. Vue compilado y API en el mismo origen. Desarrollo con Vite/proxy autorizado. Cuentas de aplicación distintas no aíslan frente a software con el mismo acceso Windows a la base/bóveda. Solo datos sintéticos mientras no se aprueben políticas reales.

**Project Type**: backend Rust y frontend Vue desacoplados en desarrollo, una aplicación local al distribuir el bundle. No microservicios, SSR, gestor de estado adicional ni proveedor LLM nuevo.

**Performance Goals**: medir búsquedas/persistencia/recuperación en datos de prueba; menos de un minuto sigue propuesto, no un SLO aprobado. No inventar latencia, carga máxima o límites de producción.

**Constraints**: stack de AGENTS.md obligatorio; una wiki por agente; asignación en toda operación; originales inmutables; dos propuestas conservadas y resolución administrativa; secretos sin salida ni fallback; intercambio solo mediado; SIPI NEXUS excluido; sin restauración general ni borrado autónomo.

**Scale/Scope**: un host, Insight/Estimador, dos programadores y administrador, proyectos Alfa/Beta y cuatro RFDs sintéticos. Son datos de prueba, no límites permanentes. SQLite serializa escritores; reevaluar almacenamiento si la carga real demuestra otra necesidad.

## Constitution Check

*Gate previo a investigación y reevaluado después del diseño. «Conforme en diseño» no acredita funcionamiento, ratificación ni políticas operativas aprobadas.*

| Principio | Antes de Phase 0 | Después de Phase 1 | Evidencia de diseño y límite |
|---|---|---|---|
| 1. Simplicidad y alcance | Conforme | Conforme | Stack Rust/Vue exigido, bibliotecas verificadas, sin funciones ajenas ni implementación en esta fase |
| 2. LLM Wiki | Conforme con obligación de separación | Conforme en diseño | OriginalSource, Revision, referencias, mantenimiento, índice derivado y WikiEvent; originales inmutables |
| 3. Aislamiento e intercambio mediado | Conforme; mediador no disponible por suposición | Conforme en diseño; aceptación externa pendiente | WikiAssignment, wiki_id e identidad verificada; adaptadores fallan cerrados, no bypass ni dependencia de SIPI NEXUS |
| 4. Trazabilidad | Conforme como requisito | Conforme en diseño | Autoría, origen/proyecto, propuestas, decisión administrativa y ExchangeReceipt sin secretos |
| 5. Contexto, no autoridad | Conforme como requisito | Conforme en diseño | Categoría epistemológica, referencias y revisión humana; texto recuperado no ejecuta reglas ni concede permisos |
| 6. Datos mínimos y secretos | Solo datos ficticios | Conforme para diseñar piloto sintético; no operación real aprobada | Bóveda local explícita, sin valores en wiki/log/sesión; políticas de secretos reales pendientes |
| Protección de datos y gobierno | Aprobaciones no acreditadas | Sin cambio; fuera de habilitación real | No se declara ratificación ni se cambian reglas de conservación/eliminación |
| Verificación | Criterios observables definidos | Contratos y guía trazados; pruebas funcionales no ejecutadas | SC-001–SC-007; SC-002 requiere servicios y mediador reales autorizados |

**Resultado de gates**: sin violaciones de diseño que requieran excepción. Está permitido documentar un piloto sintético que no elude las restricciones. No se habilita información real, secretos reales ni aceptación de intercambio hasta cumplir las dependencias. Si implementación propone eludir mediación, guardar valores en la base o publicar datos reales sin políticas, el gate debe fallar, no registrarse como riesgo aceptado automáticamente.

## Project Structure

### Documentation (this feature)

```text
specs/001-sipibrain-collective-memory/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── application.md
│   └── agent-exchange.md
└── checklists/requirements.md
```

`tasks.md` pertenece a `$speckit-tasks`; no se crea aquí. La especificación y constitución no se modifican en planificación. Las notas históricas del checklist permanecen sin reescribir; sus marcas actuales son una evaluación documental, no prueba de aplicación.

### Source Code (repository root)

**Estructura propuesta, todavía inexistente:**

```text
backend/
├── Cargo.toml
├── Cargo.lock
├── migrations/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── api.rs
│   ├── auth.rs
│   ├── access.rs
│   ├── models.rs
│   ├── db.rs
│   ├── services.rs
│   ├── integrations.rs
│   └── windows_vault.rs
└── tests/
    ├── authorization.rs
    ├── persistence.rs
    ├── conflicts.rs
    ├── contracts.rs
    ├── windows_vault.rs
    └── live_agents.rs
frontend/
├── package.json
├── package-lock.json
├── tsconfig.json
├── vite.config.ts
└── src/
    ├── main.ts
    ├── App.vue
    ├── api.ts
    ├── types.ts
    ├── views/
    ├── components/
    └── tests/
```

**Structure Decision**: los servicios Rust centralizan autorización, escritura, referencias y resolución; ni Vue ni los adaptadores de agentes reimplementan las reglas. SQLx gestiona persistencia/migraciones; la API utiliza DTO serde y el cliente DTO TypeScript. La frontera windows_vault encapsula el pequeño bloque unsafe de la API Windows para probarlo y revisarlo por separado, sin criptografía casera. Vue usa estado local/composición suficiente para las vistas, sin biblioteca global no justificada. El backend sirve únicamente el bundle compilado, sin convertir errores `/api` en fallback HTML.

## Phase 0 — Research

Actualizada mediante investigación delegada de solo lectura y comprobación puntual de fuentes oficiales, dentro del stack obligatorio del AGENTS.md actual. [research.md](research.md) registra decisión, motivo, alternativas y evidencia para plataforma, acceso, almacenamiento de wikis, concurrencia, bóveda e intercambio. No se instaló ningún paquete ni se contactaron los agentes. Las versiones exactas se fijarán al implementar y comprobar compatibilidad, no se inventa un lockfile validado.

## Phase 1 — Design & Contracts

- [data-model.md](data-model.md): entidades, validaciones, pertenencia a wiki, fuentes/revisiones, conflicto y transiciones de bóveda.
- [contracts/application.md](contracts/application.md): interfaz local propuesta, sesiones, asignaciones, guardado, resolución y creación de secretos; no endpoints supuestos de otros agentes.
- [contracts/agent-exchange.md](contracts/agent-exchange.md): adaptadores internos propuestos, conexión/autorización/recuperación diferenciadas y fallos cerrados.
- [quickstart.md](quickstart.md): preparación futura con Cargo/npm, pruebas Rust/Vue y condiciones de aceptación real; comandos señalados como aún no implementados.

### Cobertura verificable

| Requisito | Diseño principal | Verificación prevista |
|---|---|---|
| FR-001 | WikiAssignment, rol administrativo, acceso centralizado | SC-006; negar wiki ajena y autoasignación |
| FR-002 | Record, fuentes, revisiones y procedencia | Aportaciones de dos programadores conservadas |
| FR-003 | SQLite local y datos inmutables | SC-001; reinicio sin cambio de contenido |
| FR-004 | Wiki/Project/Client y filtros de ámbito | Separación por agente y clientes homónimos |
| FR-005 | Tipo rfd, proyecto obligatorio, cliente cuando corresponde | Cuatro RFDs individuales, sin formato documental supuesto |
| FR-006 | Contrato de intercambio y recibos | SC-002 real; también puerto disponible con denegación sin contenido |
| FR-007 | Búsqueda autorizada por texto, agente y cliente | SC-003; resultados sin filtración de títulos/conteos ajenos |
| FR-008 | Detalle con origen, proyecto/cliente y referencias | Mismo RFD y procedencia en los tres tipos de búsqueda |
| FR-009 | Recuperación persistente independiente de conversación | SC-002/003; sin restauración de borrados/versiones |
| FR-010 | Bóveda Windows desde Rust con CRED_PERSIST_LOCAL_MACHINE y reservas | Crear ficticio, reiniciar y confirmar sin eco del valor |
| FR-011 | Frontera de secretos sin rutas de revelado | SC-005; inspección de DB/trazas/contexto y fallos del backend |
| FR-012 | Ausencia explícita, sin generación de antecedentes | SC-004 |
| FR-013 | Propuestas, conflictos, generación y resolución administrativa | SC-007; preservar ambas, negar resolución no autorizada/obsoleta |

### Dependencias que impiden afirmar aceptación

1. Contratos, direcciones/puertos y autorización de prueba de Insight y Estimador, canal de identidad y mediador verificable. Sin ellos, implementar núcleo/dobles y fallo cerrado; no programar protocolos ficticios ni aprobar SC-002.
2. Permisos pendientes de uso/revelado/modificación/borrado de secretos, respaldo y prevención de entrada accidental. No se añaden esas operaciones ni se prueba con valores reales.
3. Responsables y reglas de datos reales, conservación/eliminación y ratificación. No se alteran fuentes ni políticas por conveniencia del piloto.
4. Dispositivo y volumen para aprobar objetivo temporal; medir sin prometer un umbral aún no acordado.

Estos límites son explícitos y compatibles con el alcance sintético aprobado. No son declaraciones de disponibilidad ni autorizaciones remotas implícitas.

## Complexity Tracking

No se solicita excepción constitucional ni cambio del stack. La separación backend/frontend viene impuesta por AGENTS.md. Las fronteras adicionales son necesarias para la bóveda del dispositivo y el intercambio mediado; los contratos reales de los agentes siguen pendientes. No se incluyen microservicios, agentes nuevos, colas, embeddings, sincronización de secretos o recuperación general de versiones.
