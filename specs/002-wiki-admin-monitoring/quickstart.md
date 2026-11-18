# Quickstart: validación de la especificación 002

**Estado:** rutas y vistas de la 002 implementadas. Verificación automatizada: Rust 32/32, Vue 16/16, typecheck y build aprobados; verificación independiente de autorización y concurrencia 7/7. Las pruebas de la interfaz se hicieron con Vue montado y respuestas simuladas; no se ejecutó la demostración manual en navegador.

## Preparar el entorno sintético

Desde la raíz, en PowerShell, con la cadena Rust fijada en `backend/rust-toolchain.toml` y Node compatible con `frontend/package.json`:

```powershell
$env:SIPIBRAIN_DATA_DIR = Join-Path ([IO.Path]::GetTempPath()) ("sipibrain-002-" + [guid]::NewGuid())
npm --prefix frontend ci
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --locked -- init-db
cargo run --manifest-path backend/Cargo.toml --locked -- bootstrap-admin
cargo run --manifest-path backend/Cargo.toml --locked -- create-user --username dev-fixture
cargo run --manifest-path backend/Cargo.toml --locked -- seed-pilot --synthetic
cargo run --manifest-path backend/Cargo.toml --locked -- serve --bind 127.0.0.1:8080 --static-dir frontend/dist
```

Abrir `http://127.0.0.1:8080` en el entorno de prueba. Mantener el mismo directorio de datos para comprobar el reinicio. Usar contraseñas sintéticas por entrada interactiva y no publicar en la red. El puerto anterior es el local vigente, no un cambio ni una prueba de puertos de la 001.

El seed por sí solo permite comprobar el selector sin candidatos. La suite de integración de la 002 debe preparar además, en una base temporal, un agente sintético activo sin wiki y dos wikis con aportaciones conocidas. Para repetir el alta manualmente se necesita un catálogo de prueba ya provisionado con ese agente; no editar datos del usuario ni añadir alta de agentes al producto. Las dos wikis sembradas sirven para la comprobación manual de asignación y supervisión.

## Ejecutar después de la implementación

```powershell
cargo test --manifest-path backend/Cargo.toml --locked --test wiki_admin
cargo test --manifest-path backend/Cargo.toml --locked
npm --prefix frontend run test -- --run
npm --prefix frontend run typecheck
npm --prefix frontend run build
```

`wiki_admin` es la suite de integración de la 002. Si un servidor ya está usando el ejecutable habitual, configura `$env:CARGO_TARGET_DIR = Join-Path $PWD "backend/target/spec002"` antes de probar sin detener dicho servidor.

| Criterio | Acción | Evidencia esperada |
| --- | --- | --- |
| SC-001 | Crear con un agente elegible, cerrar el servidor, iniciar con los mismos datos y asignar la wiki mediante «Manage assignments». | Una sola fila persistida; el programador obtiene acceso solo a la wiki asignada. |
| SC-002 | Intentar alta sin rol o sin CSRF, con agente no elegible, con fallo de guardado y con dos solicitudes para el mismo agente. | Denegación o error explícito según contrato; cero falsos éxitos y como máximo una wiki. |
| SC-003 | Guardar un registro y dos fuentes en una wiki; actualizar; guardar una revisión sin agregar entidades. | Cambios +1 y +2 solo en esa wiki; la revisión cambia actividad, no totales. Referenciar una fuente dos veces no la duplica. |
| SC-004 | Comparar wiki nueva y fallo de carga controlado; consultar la supervisión con un programador y usar datos sintéticos identificables para verificar exclusiones. | Estado vacío distinto del error. No hay datos administrativos para el programador ni cuerpos, secretos o referencias en la supervisión. |
| SC-005 | Con dos wikis con actividad, dar al administrador asignación a una. Consultar la vista y la respuesta directa, intentar abrir ambas y luego revocar el acceso. | Fechas en ambas; autoría y contenido solo de la asignada. La siguiente consulta tras revocar conserva fechas, sin nombres ni IDs de autor. Las rutas de contenido siguen denegando el acceso. |

Para códigos de respuesta y campos esperados, ver [contrato](contracts/wiki-admin.md); para criterios de conteo y actividad, ver [modelo](data-model.md).
