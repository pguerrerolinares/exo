# Adenda al brief de redacción — ciclo de arreglo (errata E8)

Se aplica **encima** de `brief-redaccion.md`. Donde choquen, manda esta
adenda.

## Cambios al contrato

1. **Firma del check:** `check.sh <workdir> <transcript> <commit_inicio>`.
   Compara siempre el estado final contra `<commit_inicio>` (por ejemplo,
   `git -C "$1" diff "$3"`), nunca contra `HEAD`: el agente puede commitear.
   En las pruebas, pasa el commit inicial real del workdir de prueba.
2. **Nada de regex sobre texto libre** (respuesta o prosa del agente) como
   criterio de cumplimiento. Si la regla solo se ve en lo que el agente
   escribe, el check es de tipo `rubrica`.
3. **Proceso sin orden estricto:** si «hacer → verificar → corregir» cumple el
   espíritu de la regla, el check lo acepta. Solo exige orden si la regla
   trata literalmente del orden.
4. **Repos excluidos:** no uses `/home/paul/Documentos/proyectos/pguerrero-music`
   ni `/home/paul/Documentos/proyectos/agent-solve-it` como fuente. Si la tarea
   los usaba, reescríbela sobre un repo sintético (`setup.sh`, determinista,
   sin red) que conserve la situación que hace importar la regla. Si no se
   puede, declárala `no-convertible` con motivo.
5. **Disco:** si usas un repo real, anota en `tarea.json` un campo `excluir`
   con los subdirectorios grandes que la tarea no necesita (se omiten de la
   copia). Nunca excluyas lo que el check o la tarea usan.

## Para cada tarea que arreglas

- Lee el hallazgo del auditor sobre esa tarea en
  `/home/paul/.cache/exo-ablacion-k/gold/auditoria-s1.md` y aplica el
  arreglo concreto que propone, o uno equivalente que cierre el mismo
  agujero.
- Añade a `pruebas/` un caso por cada falso aprobado o falso suspenso que
  señaló el auditor, y demuestra que ahora sale bien.
- Vuelve a ejecutar todas las pruebas y actualiza `sanity`.
- Anota en `tarea.json` un campo `arreglo_e8` con una línea: qué cambió y
  por qué.
