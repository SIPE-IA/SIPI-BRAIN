# Quickstart: validación de la especificación 003

**Estado:** alcance local implementado y verificado con datos sintéticos. La aceptación con agente SIPI real (SC-004) y el cierre constitucional (T025) siguen pendientes. Usar la cadena Rust fijada en `backend/rust-toolchain.toml`, Node compatible con `frontend/package.json` y una base temporal. No sembrar agentes para el escenario de instalación vacía.

## Validación local tras implementar

Desde la raíz del repositorio, en PowerShell:

```powershell
$env:SIPIBRAIN_DATA_DIR = Join-Path ([IO.Path]::GetTempPath()) ("sipibrain-003-" + [guid]::NewGuid())
npm --prefix frontend ci
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --locked -- init-db
cargo run --manifest-path backend/Cargo.toml --locked -- bootstrap-admin
cargo run --manifest-path backend/Cargo.toml --locked -- serve --bind 127.0.0.1:8080 --static-dir frontend/dist
```

Antes del arranque normal, comprobar que `SIPIBRAIN_LOCAL_FIXTURE_CATALOG` no esté definida y no usar la opción de compilación del doble. Abrir `http://127.0.0.1:8080`, iniciar sesión y comprobar que el panel no ofrece destinos sintéticos ni permite introducir destinos libres. El alta de agente y la creación de wiki siguen disponibles. Para el recorrido desde una instalación vacía, no crear agentes en esta comprobación inicial. Detener el backend con Ctrl+C antes de arrancarlo con el doble.

```powershell
cargo test --manifest-path backend/Cargo.toml --locked
npm --prefix frontend run test -- --run
npm --prefix frontend run typecheck
```

| Criterio | Acción | Resultado esperado |
| --- | --- | --- |
| SC-001 | Desde el panel registrar el primer agente, crearle una wiki antes de verificar conexión, seleccionar un destino de prueba aprobado, guardar y probar; recargar y reiniciar. | Agente, wiki y configuración persisten; no se necesita editar el backend durante el flujo. El resultado efímero no reaparece al recargar. |
| SC-002 | Enviar dos altas simultáneas con el mismo ID y dos altas de wiki para el agente. Repetir las cuatro mutaciones/pruebas sin rol y comprobar CSRF ausente con sesión administradora. | Una fila por entidad y conflictos explícitos; solicitudes sin autorización no escriben ni contactan. El administrador sin asignación no puede leer el contenido. |
| SC-003 | Ejecutar los cuatro casos manuales del doble: identidad válida, plazo agotado, identidad inválida y aprobación retirada después de guardar. | Solo la identidad válida da éxito. El último caso no realiza contactos. Ningún caso usa contenido ni cambia asignaciones. |

### Doble local optativo para el panel

El doble y el backend de prueba solo escuchan en `127.0.0.1`. El doble usa identidades sintéticas, sin credenciales reales ni contenido de wikis. El backend solo acepta el doble con la opción de compilación `local-connection-fixture` **y** la ruta explícita al catálogo temporal; ninguno se habilita por defecto.

1. En la terminal donde preparaste el backend, conserva `SIPIBRAIN_DATA_DIR` y obtén la ruta del catálogo. Copia la ruta absoluta a una segunda terminal, abierta en la raíz del repositorio.

```powershell
$catalog = Join-Path $env:SIPIBRAIN_DATA_DIR 'local-connection-catalog.json'
$catalog
```

2. En la segunda terminal, asigna a `$catalog` la ruta copiada y arranca el doble. Reserva los puertos 18081–18083 para los tres destinos y el 18084 de loopback para serializar las revocaciones locales. Espera a que el doble indique que publicó el catálogo inicial.

```powershell
node scripts/local-connection-fixture.cjs serve --port 18081 --catalog "$catalog"
```

3. En la terminal del backend, activa el catálogo solo para esa sesión y arranca el backend. Abre el panel en `http://127.0.0.1:8080` con la cuenta de prueba.

```powershell
$env:SIPIBRAIN_LOCAL_FIXTURE_CATALOG = $catalog
cargo run --manifest-path backend/Cargo.toml --locked --features local-connection-fixture -- serve --bind 127.0.0.1:8080 --static-dir frontend/dist
```

4. Registra el agente desde el panel. En cada caso, selecciona el destino aprobado, guarda la identidad esperada `fixture-agent` y pulsa **Test connection**. Para SC-001, crea su wiki antes de probar y usa el destino `fixture-ok`; para comprobar persistencia, reinicia el backend del paso 3 conservando el mismo directorio temporal y el catálogo, con el doble aún en ejecución.

| Destino | Resultado esperado |
| --- | --- |
| `fixture-ok`: identidad correcta | `verified` |
| `fixture-slow`: respuesta posterior al plazo | `unreachable` |
| `fixture-wrong`: identidad distinta | `identity_unverified` |
| `fixture-ok` después de revocar su aprobación | `not_testable`, sin aumentar el contador de contactos del doble |

5. Para el último caso, guarda `fixture-ok` antes de revocarlo. Anota su contador de contactos en la salida del doble y ejecuta el siguiente comando desde otra terminal, con la misma ruta de catálogo. Espera la confirmación del backend de que activó la nueva versión, **sin reiniciarlo**. Comprueba que el destino ya no aparece entre los elegibles, que la configuración guardada sigue visible como no comprobable y que no hay un nuevo contacto al intentar la prueba. Si el panel deshabilita el botón, T012 verifica la respuesta `not_testable` en la API.

```powershell
node scripts/local-connection-fixture.cjs revoke --catalog "$catalog" --destination fixture-ok
```

Si una revocación informa que el puerto de bloqueo 18084 está ocupado, esa operación no se publicó. Reinténtala cuando quede libre. Una terminación forzada del proceso libera el puerto; no hay que eliminar ningún archivo de bloqueo para volver a revocar.

Al terminar, detén ambos procesos con Ctrl+C y quita `SIPIBRAIN_LOCAL_FIXTURE_CATALOG` de la terminal del backend. Este recorrido no acredita SC-004.

### Regresiones automatizadas (T012 y T013)

Estas comprobaciones se ejecutan en las suites indicadas arriba; no son pasos del recorrido manual del doble.

- **T012 (backend):** comprobar que no se siguen redirecciones; que los cambios de revisión o aprobación durante la prueba no permiten confirmar un éxito obsoleto; y que un fallo de identidad conserva el agente, su wiki y su asociación.
- **T013 (frontend):** comprobar que el panel no muestra como actual una respuesta fuera de orden, de otra revisión o de otro agente, ni restaura el resultado tras recargar.

### Evidencia local

- Línea base de la 002: 4 pruebas Rust de administración de wikis y 1 de Vue pasaron antes de editar. El alta y la conexión nuevas mostraron RED antes de conectar sus rutas y vistas.
- Rust: 35 pruebas de la suite normal y 8 pruebas focalizadas con `local-connection-fixture` (alta de agente, administración de wikis y conexión) pasaron. Vue: 20 pruebas, comprobación de tipos y build pasaron. Se usó Rust 1.98.0 y Node 24.22.1; en Windows el ejecutable habitual estaba en uso, por lo que Rust se ejecutó con `CARGO_TARGET_DIR=backend/target/spec003` sin detener procesos ajenos.
- Navegador Chromium con SQLite temporal: alta desde cero, creación de wiki, configuración persistida tras reinicio y resultado anterior no restaurado. Identidad correcta, distinta y plazo agotado mostraron resultados distintos. La revocación se activó sin reiniciar, conservó la configuración, devolvió `not_testable` en la API y produjo cero contactos adicionales.
- El doble permaneció deshabilitado en los cuatro arranques negativos de T024, con cero contactos. No se contactaron servicios SIPI reales.

## Aceptación real (SC-004)

Antes de ejecutar, obtener del responsable el contrato compatible, destino actualmente aprobado, método autorizado de comprobación de identidad y autorización explícita para la operación y el canal de autenticación concreto. No descubrir destinos ni reutilizar credenciales ambientales. Con esa aprobación y el adaptador implementado, seleccionar el destino en el panel y ejecutar **Test connection**. Registrar fecha, entorno, referencia del contrato, resultado y evidencia no sensible de que no hubo consultas de contenido ni cambios de permisos.

**Pendiente de evidencia:** sin esos insumos o sin una verificación exitosa desde el panel con un SIPI real, SC-004 queda bloqueado. Los dobles, un puerto accesible y una respuesta HTTP sin identidad comprobada no lo aprueban.

## Evidencia de cierre constitucional

T025 se completa solo con evidencia aplicable al candidato para todos los controles y con SC-004 verificado en T022. Si falta algún resultado, informar el avance parcial sin cerrar la entrega. Referenciar pruebas existentes cuando su evidencia sea aplicable al candidato; no implementar intercambio como parte de la 003.

| Control | Evidencia esperada | Resultado y referencia |
| --- | --- | --- |
| Persistencia y recuperación | Tras reiniciar, recuperar el agente, la wiki y su configuración; comprobar que una fuente o registro de prueba sigue accesible para el usuario asignado. | Pendiente. |
| Separación | El usuario asignado puede leer la wiki; el usuario no asignado, incluso si es administrador, no puede leer su contenido. | Pendiente. |
| Trazabilidad | Un registro de prueba conserva origen, agente responsable y contexto. La evidencia no expone secretos ni contenido ajeno. | Pendiente. |
| Intercambio permitido y denegado | Vincular la evidencia de producto de la 001 cuando se haya ejecutado y sea aplicable al candidato. SC-004 y los dobles no sustituyen esta evidencia. | Bloqueado: [T038 y T052 de la 001](../001-sipibrain-collective-memory/tasks.md) siguen pendientes. |

Ver [contrato API](contracts/agent-onboarding.md) y [reglas de estado](data-model.md).
