---
permalink: "{{KB_NAME}}/learnings/fallo-silencioso"
title: El fallo más caro es el que no avisa, y cada forma tiene su remedio
tags: [agentes, verificacion, calidad, gates]
tier: stable
semilla: true
---

# El fallo más caro es el que no avisa, y cada forma tiene su remedio

Un fallo que sale con un mensaje de error, un test en rojo o un proceso con
código distinto de cero se detecta solo: alguien lo ve y actúa. El peligroso
es el otro: el que tiene forma válida, no dispara ninguna alarma y se descubre
mucho después, por casualidad. **El modo de fallo más caro no es el que
revienta, es el que devuelve algo plausible**: un crash se arregla en una
tarde; una respuesta verosímil se cuela en el índice, el informe y la
decisión. Los mecanismos son seis y no se defienden igual.

## 1. Degradar devolviendo algo válido es corromper

Si el camino de emergencia produce un resultado con la misma forma que el
bueno, el consumidor no puede distinguirlos. Ejemplos: un indexador que cae a
un vector de hash cuando falla el embedding y mete basura sintácticamente
válida; una respuesta legítima pero vacía que sobrescribe un snapshot
válido con un vacío; una búsqueda "híbrida" que sin la tabla de vectores devuelve
solo léxica pero etiquetada `hybrid`.

- **Todo camino de degradación emite señal en el canal que el consumidor ya
  lee** (campo en el envelope, línea visible en stdout). Si no, no es un
  camino: es un agujero.
- **Reparte por criticidad**: lo interactivo degrada con línea visible; lo
  offline falla fuerte.
- **`null` > dudoso.** Un campo nulo obliga al humano a completar; uno
  incorrecto corrompe en silencio.

## 2. Un check debe poder salir rojo por su caso y verde cuando todo va bien

Las dos direcciones; si falla una, es teatro. Ver también
[[Un negativo no vale si el instrumento no está validado|learnings/instrumento-validado-antes-de-medir]].

- **La prueba definitiva de un gate es la mutación, no el color.** Una suite en
  verde toleró que se borraran dos invariantes enteros de la spec. Rompe el
  código a propósito y exige que el test caiga.
- **Un gate con varias causas, demostrado por una sola, no está demostrado
  para las otras.** Un CI visto rojo por el formateo se dio por falsable, pero
  los pasos de un job son secuenciales: el linter quedaba `skipped` detrás y
  los jobs de test no habían fallado nunca.
- **El patrón del gate debe ser más ancho que la regla que vigila**, nunca
  igual. Un gate de privacidad buscaba el mismo patrón estrecho que la regla de
  sustitución: la variante con otro separador sobrevivía a las dos.
- **Un guard sin vía de excepción es un guard muerto.** Los falsos rojos
  permanentes acaban ignorados; todo rechazo lleva un `--force` registrado.
- **Un control que no se ejecuta sobre el target es documentación, no
  enforcement.** Un hallazgo real: los controles que cazaban un falso negativo
  existían en el repo y nadie los corría en la plataforma donde fallaba.
- **El caso que se evapora.** No que el test falle mal: que deje de existir
  sin que el contador lo note. Un abort fatal saltaba el resto de la
  función, el caso no llegaba a registrar ni éxito ni fallo, y la suite salía
  en verde con un caso menos. Remedio: capturar el rc del sujeto en
  subshell y un **backstop de casos ejecutados vs esperados**; un instrumento
  que no se cuenta a sí mismo no sabe cuándo le falta un trozo.
- **Un valor hardcodeado no mide, recita.** Un contador o resultado escrito a
  mano, o leído de un fichero en vez de obtenido ejecutando, siempre cuadra.
  Infradeclarar es tan falso como exagerar y se cuela porque parece
  prudencia. Remedio: contadores derivados del propio resultado.

Antes de confiar en un check, ejecútalo contra un caso roto de verdad y contra
el sano. Si no sabes construir el roto, tienes una decoración.

## 3. Contrato por prosa o nombre duplicado se rompe mudo

- Un router que decide leyendo prosa renderizada (un substring): reescribir el
  texto humano rompe el routing, y el test que hardcodea la misma frase pasa
  igual.
- Dos detectores del mismo concepto con tolerancias distintas (uno exige
  `\n---\n` exacto, otro no): pérdida silenciosa en el borde.

**Un solo detector por concepto, o un test de acoplamiento que falle cuando
diverjan. Si el contrato viaja como texto, que viaje como identificador
estable, no como frase.** Primo hermano en
[[Una regla que se cita y se ignora es un comentario|learnings/la-prosa-no-es-enforcement]].

## 4. Exit 0 mide terminación, no efecto

Un job con una dependencia caducada terminaba en exit 0 (degradación
elegante) sin haber producido nada. **Verifica contra el estado** (un
`select count(*)`, el fichero en disco), no contra la salida de la operación.
Con cron o procesos detached, **prueba con `env -i`**: un `jq` invisible en el
PATH lo mata sin ruido. Visión general en
[[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]].

## 5. Dos cambios correctos pueden componer un fallo

Un refresco de datos y una política de retención eran correctos por
separado; juntos borraban datos reales. El fallo no estaba en ningún cambio,
estaba entre dos. **Cuando dos cambios tocan el mismo estado,
la prueba es la composición**; para eso existe la review whole-branch de
`exo:orchestrate`, que ve lo que ninguna review de tarea puede ver.

## 6. Ausencia de hallazgos no es evidencia de ausencia

Un sweep que devuelve filas parece exhaustivo porque trae datos; las
construcciones que el analizador no entiende no dan error, simplemente no
existen.

- **Censo antes que muestreo**: comprueba que el conjunto de lo visto es
  exactamente el esperado, no solo que traiga filas.
- **Telemetría de todo "unresolved" con su razón**: así el hueco tiene nombre
  y tamaño en vez de ser un cero.

## Cómo se caza en la práctica

1. **Ejecuta, no releas el diff.** Simular una entrada vacía y ver
   vaciarse un snapshot encontró lo que leer el código no.
2. **Pregunta por el caso vacío**: la mitad de estos fallos son un conjunto
   vacío pasando por bueno.
3. **Borra la línea que el test dice cubrir** y mira si se pone rojo. Barato y
   rentable: un test que no puede fallar es lo habitual cuando se escribe
   contra código que aún no existe, sea quien sea quien lo escriba.
4. **El nombre del test es una afirmación** y puede ser falsa: un test que
   promete "conserva la fila de mejor rank" sobre un fixture con dos filas
   empatadas lo decide un tie-break no garantizado. Si el fixture no sostiene
   el nombre, se renombra el test; no se escribe el assert.
5. **Dale la lente al revisor**: "el fallo típico aquí no es un error de lógica
   visible, es una desalineación silenciosa entre lo que un artefacto declara
   y lo que hace".

## Cuándo no aplica

Prototipos desechables y código sin consumidor aguas abajo: no merece un
backstop ni un `--force` registrado. El coste del rigor crece con lo que
depende de la salida; para un fallo que sí grita (excepción, rojo, exit ≠ 0),
basta lo normal.
