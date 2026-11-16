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

Las reglas de producto viven en .specify/memory/constitution.md y el estado del producto en specs/README.md
