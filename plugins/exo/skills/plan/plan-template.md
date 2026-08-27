# Plantilla de plan

**Cuándo usar:** al escribir el plan completo, para el header obligatorio y
la estructura de cada tarea. Destilado de `writing-plans/SKILL.md`
(superpowers 6.1.1, MIT © 2025 Jesse Vincent).

## Header obligatorio

Todo plan empieza así:

```markdown
# [Nombre de la feature] Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: usa `exo:orchestrate`
> para ejecutar este plan tarea a tarea. Los pasos usan checkbox (`- [ ]`)
> para tracking.

**Goal:** [una frase describiendo qué construye este plan]

**Architecture:** [2-3 frases sobre el enfoque]

**Tech Stack:** [tecnologías/librerías clave]

## Global Constraints

[Los requisitos project-wide de la spec — version floors, límites de
dependencias, reglas de naming y copy, requisitos de plataforma — una línea
cada uno, con valores exactos copiados verbatim de la spec. Los requisitos
de toda tarea incluyen implícitamente esta sección.]

---
```

## Estructura de tarea

````markdown
### Task N: [Nombre del componente]

**Files:**
- Create: `path/exacto/al/fichero.py`
- Modify: `path/exacto/al/existente.py:123-145`
- Test: `tests/path/exacto/al/test.py`

**Interfaces:**
- Consumes: [qué usa esta tarea de tareas anteriores — firmas exactas]
- Produces: [qué usan tareas posteriores — nombres de función, tipos de
  parámetro y retorno exactos. El implementer de una tarea solo ve su
  tarea; este bloque es cómo aprende los nombres y tipos vecinos]

- [ ] **Step 1: Escribir el test que falla**

```python
def test_specific_behavior():
    result = function(input)
    assert result == expected
```

- [ ] **Step 2: Correr el test y verificar que falla**

Run: `pytest tests/path/test.py::test_name -v`
Expected: FAIL con "function not defined"

- [ ] **Step 3: Implementación mínima**

```python
def function(input):
    return expected
```

- [ ] **Step 4: Correr el test y verificar que pasa**

Run: `pytest tests/path/test.py::test_name -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/path/test.py src/path/file.py
git commit -m "feat: add specific feature"
```
````

## No-placeholders (lista literal)

Cada paso debe contener el contenido real que el ingeniero necesita. Esto son
**fallos de plan** — nunca los escribas:

- "TBD", "TODO", "implement later", "fill in details"
- "Add appropriate error handling" / "add validation" / "handle edge cases"
- "Write tests for the above" (sin el código del test)
- "Similar to Task N" (repite el código — el ingeniero puede leer las tareas
  fuera de orden)
- Pasos que describen qué hacer sin mostrar cómo (bloques de código
  obligatorios en pasos de código)
- Referencias a tipos, funciones o métodos no definidos en ninguna tarea
