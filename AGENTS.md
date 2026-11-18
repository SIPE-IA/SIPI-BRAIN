# AGENTS.md — SIPI-brain

## Propósito
Gestor de LLM wikis para agentes de IA: memoria persistente por agente,
con fuentes originales, conocimiento sintetizado, referencias, índice e historial.

## Stack
Backend: Rust. Frontend: Vue 3 + TypeScript.
Este stack es la instrucción vigente del usuario; el plan de
specs/001-sipibrain-collective-memory/plan.md describe backend Rust y frontend Vue 3 + TypeScript.

## Desarrollo local
La base ejecutable está en backend/ (Cargo) y frontend/ (npm/Vite).
El quickstart de specs/001-sipibrain-collective-memory/ distingue comandos y resultados
comprobados de las historias pendientes. El estado por tarea y su evidencia viven en
specs/001-sipibrain-collective-memory/tasks.md; no presentar tareas pendientes como verificadas.

## Convenciones
- Preferir cambios mínimos, reutilizar patrones y no ampliar alcance sin permiso.
- Código, identificadores y UI en inglés; documentación técnica en español profesional.
- Conservar fuentes originales; distinguir hechos, decisiones y suposiciones.
- Aislar wikis por agente; compartir solo mediante mediación autorizada.
- No guardar secretos en código, memoria, prompts ni registros.
- Verificar cambios con pruebas y declarar comprobaciones pendientes.
- Commits convencionales, sin atribución a IA; Makefiles con targets canónicos y alias universales.

## Cierre de una feature

Solo cuando el usuario pida explícitamente «cerrar la feature», ejecutar en este orden:

1. Revisar `git status`. Si hay cambios pendientes de la feature, hacer los commits correspondientes y confirmar que el árbol de trabajo quede limpio. Si quedan cambios ajenos, detenerse sin incluirlos.
2. Ejecutar las pruebas del proyecto. Si alguna falla o no se puede ejecutar, detenerse antes de cambiar de rama.
3. Guardar el nombre de la rama de la feature y ejecutar `git checkout main`. Si `main` no existe localmente, detenerse sin sustituirla por otra rama.
4. Ejecutar `git merge --no-ff -m "merge feature NNN:<nombre>" <rama-feature>`, sustituyendo `NNN` y `<nombre>` por los valores de la feature. Este mensaje es la excepción al formato de commits convencionales. Si el merge falla, no borrar la rama.
5. Borrar solo la rama local de la feature con `git branch -d <rama-feature>`, sin forzar.
6. Mostrar la salida de `git log --oneline --graph -10`. No hacer push como parte del cierre.

Las reglas de producto viven en .specify/memory/constitution.md y el estado del producto en specs/README.md
