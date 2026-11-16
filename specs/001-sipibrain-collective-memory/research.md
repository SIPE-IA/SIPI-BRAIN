# Research: SIPI-brain — Rust y Vue 3 + TypeScript

**Fecha**: 2026-11-16. Corrección del plan para cumplir el [AGENTS.md del proyecto](../../AGENTS.md), sección Stack. Ese archivo es la fuente del stack obligatorio: backend Rust y frontend Vue 3 + TypeScript. La investigación elige bibliotecas dentro de esa restricción, no un lenguaje alternativo. No se instalaron dependencias ni se probó código.

## 1. Aplicación y herramientas

- **Decisión**: backend Rust con Axum 0.8, Tokio 1 y serde 1; frontend Vue 3 + TypeScript con Vite. Backend sirve `/api` y el bundle compilado del frontend bajo un origen local; en desarrollo Vite utiliza proxy explícito hacia la API. Usar `tower-http` para archivos estáticos, no un servidor de archivos artesanal. No añadir SSR, estado global adicional, proveedor LLM o base vectorial.
- **Motivo**: cumple el stack indicado y mantiene un único backend de dominio; separa las dos compilaciones sin crear microservicios.
- **Alternativas consideradas**: un framework completo distinto del stack está descartado por instrucción del usuario. API y UI en orígenes públicos separados añadirían despliegue y CORS no necesarios para el piloto.
- **Evidencia**: [Axum](https://docs.rs/axum/0.8.4/axum/), [Tokio](https://docs.rs/tokio/latest/tokio/), [serde](https://docs.rs/serde/latest/serde/), [Vue y TypeScript](https://vuejs.org/guide/typescript/overview), [proxy de Vite](https://vite.dev/config/server-options#server-proxy), [servicio estático de tower-http](https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html).
- **Versiones**: las familias siguientes son una línea base documentada, no un lockfile probado. Fijar Rust/Cargo y Node/npm según los requisitos de las versiones elegidas y una compilación real; no se inventa un mínimo numérico. Entregar Cargo.lock y package-lock.json al implementar.

## 2. Persistencia y sesiones compatibles

- **Decisión**: SQLx 0.8.6 con SQLite y runtime Tokio; tower-sessions 0.14.0 con tower-sessions-sqlx-store 0.15.0 y feature SQLite. Sesiones persistentes en servidor; no MemoryStore para probar reinicios.
- **Motivo**: el manifiesto publicado de ese almacén declara SQLx 0.8 y session-core 0.14; evita combinar familias incompatibles. SQLite sigue siendo adecuado al piloto local de un escritor, no una promesa de escalabilidad ilimitada.
- **Alternativas consideradas**: memoria volátil incumple continuidad de sesión; mezclar automáticamente la última versión de cada crate sin revisar sus dependencias no verifica compatibilidad. PostgreSQL añade un servicio no solicitado.
- **Evidencia**: [SQLx 0.8.6](https://docs.rs/sqlx/0.8.6/sqlx/), [manifiesto publicado del almacén 0.15.0](https://docs.rs/crate/tower-sessions-sqlx-store/0.15.0/source/Cargo.toml), [tower-sessions 0.14.0](https://docs.rs/tower-sessions/0.14.0/tower_sessions/).

## 3. Autenticación, permisos y frontend

- **Decisión**: usuarios locales con hash Argon2id mediante RustCrypto argon2 0.6, salt generado con aleatoriedad del sistema y formato PHC de la biblioteca. Crear administrador interactivamente, sin contraseña predeterminada. La API valida sesión, rol y WikiAssignment en cada operación; los controles Vue no autorizan operaciones.
- **Motivo**: cambiar de framework no elimina la obligación de autenticar ni de comprobar asignaciones. Reutilizar hashing y gestión de sesiones evita algoritmos propios.
- **Sesión y CSRF**: rotar ID al iniciar sesión, invalidarlo al salir, cookies HttpOnly host-only con SameSite explícito y caducidad. Token CSRF ligado a sesión y validación exacta de Origin en mutaciones, incluido login/logout. No usar SameSite como única protección. Sesiones sin valores secretos. Secure exige HTTPS correctamente configurado; la excepción HTTP local es exclusivamente sintética y no habilita producción.
- **UI**: Vue muestra texto escapado y referencias; no usar `v-html` con conocimiento arbitrario, ni guardar valores secretos en estado persistido, localStorage o consola. La API sigue denegando aunque se manipule la UI.
- **Alternativas consideradas**: contraseñas planas, autenticación por nombre recibido, tokens inventados y permisos solo en frontend quedan descartados.
- **Evidencia**: [RustCrypto Argon2](https://docs.rs/argon2/0.6.0/argon2/), [sesiones](https://docs.rs/tower-sessions/0.14.0/tower_sessions/), [CSRF](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html), [seguridad Vue](https://vuejs.org/guide/best-practices/security).

## 4. LLMwiki y ediciones concurrentes

- **Decisión**: mantener una wiki lógica por agente, fuentes originales inmutables, conocimiento Markdown, referencias, reglas de revisión humana, índice derivado y eventos cronológicos. SQLx aplica autorización y guardado en transacciones cortas BEGIN IMMEDIATE sobre una única conexión retenida durante la transacción, con rollback seguro ante error/cancelación.
- **Motivo**: conservar el modelo del producto, no sustituirlo al cambiar de tecnología. No hay sincronización duplicada entre archivos Markdown y tablas ni necesidad de embeddings para las búsquedas pedidas.
- **Concurrencia**: cada guardado lleva base y submission_id; el primero sin conflicto puede avanzar el head. El segundo desde una base obsoleta conserva su propuesta y la rama anterior, abre conflicto y deja de presentar un ganador definitivo. El administrador resuelve contra la generación actual; ambas propuestas y la decisión permanecen.
- **Fallo**: SQLite tiene un escritor a la vez; acotar espera/reintentos de la transacción completa con el mismo ID. Si se agotan, no afirmar guardado y conservar el texto ordinario para reintento. No mantener la transacción durante llamadas de red, bóveda o decisiones humanas.
- **Alternativas consideradas**: último guardado gana pierde trabajo; descartar la propuesta obsoleta incumple la opción B; aprobación administrativa de toda edición añade alcance.
- **Evidencia**: [transacciones SQLite](https://sqlite.org/lang_transaction.html), [modelo de datos](data-model.md), [constitución](../../.specify/memory/constitution.md).

## 5. Bóveda Windows desde Rust

- **Decisión**: adaptador pequeño con bindings Microsoft windows-sys 0.61.2 a Windows Credential Manager; CRED_TYPE_GENERIC y persistencia explícita CRED_PERSIST_LOCAL_MACHINE. Errores, tamaños, buffers y liberación mediante CredFree controlados en una frontera aislada. No implementar criptografía propia ni seleccionar backends alternativos.
- **Motivo**: el requisito es permanencia en el dispositivo, no roaming. El backend Windows de keyring-rs 3.6.3 fija CRED_PERSIST_ENTERPRISE; habilitar windows-native no demuestra la persistencia requerida. No se utiliza esa crate como sustitución automática.
- **Creación exclusiva en la aplicación**: reservar nombre único por creador y target opaco en SQLite antes de llamar a CredWriteW, que puede crear o modificar. Confirmar después. Un fallo entre ambos almacenes queda indeterminado, sin repetición ciega ni falsa atomicidad. La protección no se extiende a software hostil con la misma cuenta Windows.
- **Límite**: solo creación y confirmación no sensible. Valores fuera de SQLite, sesiones, JSON de salida, logs, errores, wikis y contexto de agentes; ningún endpoint de revelar, cambiar, borrar, exportar o ejecutar. Solo valores ficticios hasta aprobar las políticas diferidas. Revisar y probar la pequeña frontera unsafe al implementar.
- **Alternativas consideradas**: backend con roaming incumple el objetivo; texto plano y cifrado casero se descartan.
- **Evidencia**: [fuente oficial keyring-rs 3.6.3](https://raw.githubusercontent.com/open-source-cooperative/keyring-rs/v3.6.3/src/windows.rs), [bindings Microsoft](https://docs.rs/windows-sys/0.61.2/windows_sys/Win32/Security/Credentials/fn.CredWriteW.html), [persistencia Windows](https://learn.microsoft.com/en-us/windows/win32/api/wincred/ns-wincred-credentialw), [semántica de CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew).

## 6. Integraciones y verificación

- **Decisión**: conservar adaptadores internos propuestos para servicios y mediador; implementar el transporte concreto solo al recibir contratos, identidades, direcciones/puertos y autorización de prueba. No inventar endpoints de Insight/Estimador ni escanear el dispositivo. SIPI NEXUS permanece excluido.
- **Motivo**: SC-002 requiere conectividad y recuperación real y autorizada de RFDs, no un mock o socket accesible. RFD significa requerimiento funcional individual asociado a su proyecto.
- **Verificación prevista**: cargo test para servicios/autorización/persistencia y concurrencia SQLite en archivo con conexiones independientes; Vitest para interacción Vue y contrato JSON; integración navegador/API, bóveda Windows y agentes reales como pruebas separadas. No declarar e2e a partir de pruebas de componentes o dobles.
- **Alternativas consideradas**: acceso directo entre wikis elude mediación; un resultado sintético no acredita el servicio externo.
- **Evidencia**: [contrato de intercambio](contracts/agent-exchange.md), [pruebas Vue](https://vuejs.org/guide/scaling-up/testing.html).

## 7. Dependencias y límites de aceptación

| Elemento | Responsable | Entrada requerida | Si falta | Evidencia para aceptar |
|---|---|---|---|---|
| Insight/Estimador | Responsables de agentes | Contratos, destinos/puertos, identidad y sesión autorizada | Adaptadores reales deshabilitados | Recuperación efectiva de los tres RFDs de Alfa, no el de Beta |
| Mediación | Responsable SIPECOM | Servicio/agente y decisiones verificables | Cero contenido ajeno | Permiso y denegación demostrados; sin dependencia de SIPI NEXUS |
| Uso real de secretos | SIPECOM y responsable del dispositivo | Uso/permisos pendientes, respaldo y prevención de entrada accidental | Valores ficticios únicamente | Política aprobada y pruebas específicas de almacenamiento local |
| Información real y gobierno | SIPECOM | Responsables, conservación/eliminación y aprobación | Solo datos sintéticos | Aprobación documentada, no ratificación simulada |
| Un minuto propuesto | Responsable del piloto | Dispositivo y volumen acordados | Medir sin declarar aprobado un umbral | Evidencia repetible en condiciones aprobadas |
| Toolchains y dependencias | Implementación | Manifiestos compatibles, lockfiles y build real | No afirmar instalación o compatibilidad total | cargo check/test y build/typecheck frontend ejecutados |

La investigación resuelve las elecciones técnicas dentro de Rust/Vue; no aprueba las políticas que el usuario dejó pendientes ni declara disponibles los servicios. El archivo de estado `specs/README.md` referido por AGENTS.md no existe actualmente; no se inventó ni modificó para ocultar esa ausencia.
