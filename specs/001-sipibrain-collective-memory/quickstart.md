# Quickstart: piloto Rust + Vue 3 + TypeScript

**Estado actual**: MVP T001–T028 implementado con Rust y Vue; verificación final y estado de revisión se registran en [tasks.md](tasks.md), sección «Evidencia de implementación MVP». Las historias T029 en adelante (agentes, búsqueda avanzada, recuperación y secretos) siguen pendientes; sus escenarios de esta guía son futuros, no funcionalidades disponibles.

Fuentes de alcance: [AGENTS.md](../../AGENTS.md), [spec.md](spec.md). Diseño: [plan.md](plan.md), [data-model.md](data-model.md), contratos de [aplicación](contracts/application.md) e [intercambio](contracts/agent-exchange.md).

## 1. Prerrequisitos locales

- Windows; Rust/Cargo 1.98.0 fijado en backend/rust-toolchain.toml y Node 24.22.1/npm 11.19.1 comprobados con builds reales. Es una versión probada, no una afirmación del mínimo compatible de toda la aplicación.
- Manifiestos y lockfiles entregados: backend/Cargo.toml + Cargo.lock; frontend/package.json + package-lock.json. Versiones compatibles de SQLx, sesiones y almacén según [research.md](research.md).
- Cuenta Windows de confianza; datos de prueba únicamente. Las cuentas de aplicación no aíslan frente a procesos con acceso equivalente a la base o bóveda del sistema.
- Directorio de datos privado por defecto `%LOCALAPPDATA%\SIPECOM\SIPI-brain\pilot`, seleccionable con SIPIBRAIN_DATA_DIR. No carpeta de red ni sincronizada. No publicar backend/UI en LAN.
- Para la historia futura de secretos: bóveda Windows con persistencia CRED_PERSIST_LOCAL_MACHINE. No implementada ni verificada en el MVP; no hay fallback a texto plano.
- Para aceptación real SC-002: contratos/direcciones/puertos/identidades y sesión de prueba autorizada de Insight y Estimador, más mediador disponible y verificable. No escanear para obtenerlos.

## 2. Compilación y preparación

Desde la raíz del proyecto, PowerShell:

```powershell
npm --prefix frontend ci
npm --prefix frontend run typecheck
npm --prefix frontend run build
cargo build --manifest-path backend/Cargo.toml --locked
cargo run --manifest-path backend/Cargo.toml -- init-db
cargo run --manifest-path backend/Cargo.toml -- bootstrap-admin
cargo run --manifest-path backend/Cargo.toml -- create-user --username dev-one
cargo run --manifest-path backend/Cargo.toml -- create-user --username dev-two
cargo run --manifest-path backend/Cargo.toml -- seed-pilot --synthetic
cargo run --manifest-path backend/Cargo.toml -- serve --bind 127.0.0.1:8080 --static-dir frontend/dist
```

Todos los subcomandos anteriores están implementados. seed-pilot exige `--synthetic`, no crea contraseñas conocidas, no asigna automáticamente acceso a usuarios y no sobrescribe aportaciones editadas. El proceso resuelve static-dir desde el directorio de trabajo de la invocación, no desde una carpeta supuesta. Contraseñas interactivas, sin valores predeterminados ni argumentos sensibles.

La siembra informa identificadores estables: wikis `pilot-insight` / `pilot-estimador`, proyectos `pilot-project-alfa` / `pilot-project-beta`, clientes `pilot-client-alfa` / `pilot-client-beta`. Usarlos en los campos de asociación del MVP; no se deducen por nombre. El índice está paginado; las búsquedas filtradas pertenecen a US3 y por ahora se rechazan explícitamente con 400.

8080 es el puerto local predeterminado de SIPI-brain, no el puerto de los otros agentes. La aplicación sirve Vue y `/api` en el mismo origen. El administrador asigna wikis a los programadores y su propio acceso a las que deba revisar; el backend valida esas asignaciones. El piloto HTTP admite solo datos sintéticos; no publicar en LAN ni introducir información real sensible.

### Desarrollo desacoplado, sin CORS general

En una terminal, backend con el bind anterior. En otra:

```powershell
npm --prefix frontend run dev -- --host 127.0.0.1
```

Vite tiene proxy `/api` a 127.0.0.1:8080. Para admitir las mutaciones desde Vite, iniciar el backend con `SIPIBRAIN_DEV_ORIGIN=http://127.0.0.1:5173` (o `http://localhost:5173`, exactamente). `SIPIBRAIN_ORIGIN` permite fijar el origen principal local exacto sin barra final; por defecto deriva del listener. No CORS general. La excepción HTTP/cookie sin Secure es exclusivamente sintética; no extrapolar a datos reales o producción.

## 3. Datos y escenarios

Crear dos wikis, Insight y Estimador. Proyecto Alfa/Cliente Alfa: RFD-001, RFD-002 y RFD-003 en Insight, tres requerimientos funcionales independientes; nota de estimación en Estimador. Proyecto Beta/Cliente Beta: RFD-004 distinto. Añadir clientes homónimos con IDs diferentes. No son formatos obligatorios ni datos reales.

| Caso | Acción | Resultado obligatorio |
|---|---|---|
| SC-001 | Guardar fuente y conocimiento, reiniciar backend y frontend | Mismo contenido y asociaciones; original inmutable |
| Asignación/SC-006 | Programador solo con Estimador intenta buscar/leer/escribir Insight manipulando API, no solo UI | Denegación sin filtración ni cambios; permitido sobre su wiki |
| Sesión/CSRF | Login/logout y mutación sin token u Origin válido | Rotación/invalidez de sesión; mutación rechazada sin cambios |
| Colaboración | Dos programadores asignados aportan prompts distintos | Ambos registros y autorías, sin mezclar wikis |
| SC-003/004 | Buscar RFD por cliente, agente y frase; luego inexistente | Mismo RFD con procedencia; ausencia explícita sin inventar |
| LLMwiki | Crear conocimiento con fuentes/referencias y revisión humana | Original intacto, índice y eventos; contenido no concede permisos |
| SC-007 | Dos ediciones desde la misma base; guardar y reiniciar | Ambas propuestas conservadas y pendientes, sin ganador automático |
| Administrador | Resolver manualmente; repetir con generación obsoleta y con usuario no administrador | Resolución trazable; rechazos sin perder candidatas ni alterar permisos |
| Idempotencia | Repetir mismo submission_id/autor/cuerpo | No se crea una tercera propuesta |
| SC-005 | Crear valor ficticio único y reabrir, probar duplicado/fallo/interrupción | Persistencia solo en bóveda local; sin eco en JSON/UI/log/SQLite/contexto; indeterminate si no se conoce el resultado |
| Frontera Vue | Mostrar texto con HTML/script como dato y provocar error de creación de secreto | Nada se ejecuta; el valor no queda en consola, estado persistente o respuesta |
| Mediación denegada | Puerto accesible con mediador ausente/denegado | Cero contenido ajeno; conexión no equivale a intercambio |

## 4. Comprobaciones automatizadas

Scripts implementados: typecheck usa vue-tsc; test usa Vitest; build/dev usan Vite. Cargo prueba SQLite temporal en archivo. Los comandos siguientes no prueban por sí solos historias futuras ni integración real de agentes.

```powershell
$env:SIPIBRAIN_TEST_DATA_DIR = Join-Path ([System.IO.Path]::GetTempPath()) ("sipibrain-tests-" + [guid]::NewGuid().ToString('N'))
cargo fmt --manifest-path backend/Cargo.toml --all -- --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --locked
npm --prefix frontend run typecheck
npm --prefix frontend run test -- --run
npm --prefix frontend run build
```

Las pruebas Rust usan SQLite en archivo temporal y conexiones SQLx independientes con sincronización para concurrencia; nunca la base del piloto. La suite ordinaria tiene dobles y no contacta servicios o bóveda reales. Vitest comprueba interacción/contrato Vue, no acredita por sí solo un navegador con backend real. Completar manualmente los flujos UI/API anteriores o añadir una prueba de navegador al implementar antes de declarar e2e.

Prueba Windows separada y explícita, con target/nombre ficticio exclusivo y pruebas marcadas para no ejecutarse por defecto:

```powershell
cargo test --manifest-path backend/Cargo.toml --test windows_vault -- --ignored
```

El informe debe acreditar que se ejecutó al menos un caso y qué comprobó; un código cero con cero pruebas no demuestra persistencia. La limpieza solo afecta el fixture ficticio de ese test, nunca credenciales existentes. El wrapper verifica internamente persistencia/buffers sin imprimir valores. Esto no crea endpoints de lectura o borrado de secretos.

## 5. Integración real — no reemplazable por mocks

Después de recibir los contratos reales, configurar SIPIBRAIN_LIVE_CONFIG con una ruta aprobada de destinos, protocolos, puertos y referencias de autorización, sin credenciales en texto. Adaptadores deshabilitados por defecto y sin reutilizar sesiones ambientales.

```powershell
cargo test --manifest-path backend/Cargo.toml --test live_agents -- --ignored
```

La invocación expresa requiere configuración y autorización, y falla/declara bloqueo si faltan; no se omite silenciosamente. Comprobar número de casos ejecutados y evidencia, no solo código de salida.

1. Respuesta e identidad de ambos servicios en sus puertos configurados, sin escaneo.
2. Conversación nueva en Estimador; solicitar antecedentes de Alfa.
3. Mediación válida y recuperación efectiva de RFD-001/002/003 desde Insight más la nota propia de Estimador, sin reintroducir antecedentes.
4. RFD-004 de Beta excluido; proyectos/procedencia intactos; recibo sin secretos.
5. Denegación/mediador ausente: no hay datos ajenos aunque el puerto responda.
6. Medir tiempo con dispositivo/volumen documentados; el minuto propuesto no es un umbral aprobado.

Sin contratos, servicios y mediador, SC-002 queda **bloqueado/no ejecutado**, no PASS. Los secretos reales siguen bloqueados por las políticas pendientes, no se habilitan por usar Rust ni almacenamiento del sistema.

## 6. Evidencia

Registrar versiones/lockfiles, entorno, comandos, código de salida y casos aprobados/fallidos/no ejecutados con referencias sanitizadas. Comparar hash del estado de negocio antes/después de rechazos; los eventos de denegación pueden crecer sin cambiar propuestas o permisos. No guardar cuerpos de secretos, volcados, credenciales o capturas con valores.

Resultado comprobado: Vue14 PASS, typecheck/build; Rust26 PASS antes de incorporar dos sondas adicionales de seed. Verificación independiente del núcleo y seed satisfactoria. Flujo real navegador/API: dos programadores aportan con autoría, fuente y revisión; propuestas desde la misma base reciben201/202; después de reiniciar se conservan sesión, aportaciones y ambas candidatas; un administrador resuelve manualmente, sin modificar el hash original ni heredar la revisión humana anterior. El servidor temporal se detuvo. La ejecución final de suites y la revisión nativa se registran por separado en tasks.md; no se infiere aprobación de revisión desde pruebas funcionales.

Bóveda y conexiones reales **no ejecutadas**. Cuatro advertencias npm moderadas transitivas del entorno de pruebas no tienen corrección compatible automática; están en dependencias de desarrollo y no se declaran resueltas. Vitest tuvo dos timeouts de arranque de worker durante trabajo paralelo; la ejecución secuenciada posterior sí ejecutó y aprobó14 casos, no se cuentan los timeouts como pruebas RED.
