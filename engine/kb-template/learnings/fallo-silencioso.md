---
permalink: "{{KB_NAME}}/learnings/fallo-silencioso"
title: El fallo más caro es el que no avisa, y cada forma tiene su remedio
tags: [agentes, verificacion, calidad, gates]
tier: stable
semilla: true
---

# El fallo más caro es el que no avisa, y cada forma tiene su remedio

Un fallo con mensaje de error, test en rojo o exit distinto de cero se detecta
solo. El peligroso tiene forma válida, no dispara ninguna alarma y se descubre
tarde, por casualidad. **El modo de fallo más caro no es el que revienta, es el
que devuelve algo plausible**: se cuela en el índice, el informe y la decisión.
Los mecanismos son siete; esta nota trata de diseñar comprobaciones que fallen
cuando deben.

## 1. Degradar devolviendo algo válido es corromper

Si el camino de emergencia produce un resultado con la misma forma que el
bueno, el consumidor no los distingue: un indexador que cae a un vector de hash
cuando falla el embedding; un snapshot válido sobrescrito por una respuesta
vacía; una búsqueda "híbrida" que sin vectores devuelve solo léxica pero
etiquetada `hybrid`.

- **Todo camino de degradación emite señal en el canal que el consumidor ya
  lee** (campo en el envelope, línea visible en stdout). Si no, es un agujero.
- **Reparte por criticidad**: lo interactivo degrada con línea visible; lo
  offline falla fuerte.
- **`null` > dudoso.** Un campo nulo obliga al humano a completar; uno
  incorrecto corrompe en silencio.

## 2. Un check debe poder salir rojo por su caso y verde cuando todo va bien

Si falla una de las dos direcciones, es teatro. La prueba de fondo es romper el
verificador a propósito: ver
[[La lectura verifica coherencia; solo la ejecución verifica verdad|learnings/verificar-ejecutando-no-leyendo]].
Lo propio del diseño de gates:

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
  enforcement.** Los controles que cazaban un falso negativo existían en el
  repo y nadie los corría en la plataforma donde fallaba.
- **El caso que se evapora.** No que el test falle mal: que deje de existir
  sin que el contador lo note. Un abort fatal saltaba el resto de la función,
  el caso no registraba ni éxito ni fallo, y la suite salía en verde con un
  caso menos. Remedio: capturar el rc del sujeto en subshell y un **backstop
  de casos ejecutados vs esperados**; un instrumento que no se cuenta a sí
  mismo no sabe cuándo le falta un trozo.
- **Un valor hardcodeado no mide, recita.** Un contador escrito a mano, o leído
  de un fichero en vez de obtenido ejecutando, siempre cuadra. Infradeclarar es
  tan falso como exagerar. Remedio: contadores derivados del propio resultado.
- **Borra la línea que el test dice cubrir** y mira si se pone rojo; escribir
  tests contra código que aún no existe produce tests que no pueden fallar.
- **El nombre del test es una afirmación** y puede ser falsa: uno que promete
  "conserva la fila de mejor rank" sobre un fixture con dos filas empatadas lo
  decide un tie-break no garantizado. Si el fixture no sostiene el nombre, se
  renombra el test; no se escribe el assert.
- **Pregunta por el caso vacío**: la mitad de estos fallos son un conjunto
  vacío pasando por bueno.

## 3. Contrato por prosa o nombre duplicado se rompe mudo

La variante de la prosa está en
[[Una regla que se cita y se ignora es un comentario|learnings/la-prosa-no-es-enforcement]].
Lo exclusivo: dos detectores del mismo concepto con tolerancias distintas
pierden datos en el borde entre ambos. **Un solo detector por concepto, o un
test de acoplamiento que falle cuando diverjan; si el contrato viaja como
texto, que viaje como identificador estable, no como frase.** Tampoco compila el texto que anuncia un comportamiento: **cambiar un default
es tocar una superficie de usuario**. Ayuda, errores y docs que describen el
valor anterior siguen en pie y ningún gate los ve; se corrigen en el mismo
cambio.

## 4. Exit 0 mide terminación, no efecto

Un job con una dependencia caducada terminaba en exit 0 (degradación elegante)
sin haber producido nada. **Verifica contra el estado** (un `select count(*)`,
el fichero en disco), no contra la salida de la operación. Con cron o procesos
detached, **prueba con `env -i`**: un `jq` invisible en el PATH lo mata sin
ruido. Hay un primo: **el hook que se apaga solo cuando falta un input**. Si en un
modo headless no llega el dato que necesita, anota el motivo en su log y termina
bien; el efecto que prometía desaparece sin rastro visible. Un canario que lo
corra en ese modo y compruebe el efecto lo caza.

## 5. Dos cambios correctos pueden componer un fallo

Un refresco de datos y una política de retención eran correctos por separado;
juntos borraban datos reales. El fallo no estaba en ningún cambio, estaba entre
dos. **Cuando dos cambios tocan el mismo estado, la prueba es la composición**;
para eso existe la review whole-branch de `exo:orchestrate`, que ve lo que
ninguna review de tarea puede ver.

## 6. Ausencia de hallazgos no es evidencia de ausencia

Un negativo solo vale si el instrumento puede ver: ver
[[Un negativo no vale si el instrumento no está validado|learnings/instrumento-validado-antes-de-medir]].

- **Censo antes que muestreo**: comprueba que el conjunto de lo visto es
  exactamente el esperado, no solo que traiga filas.
- **Telemetría de todo "unresolved" con su razón**: así el hueco tiene nombre
  y tamaño en vez de ser un cero.

## 7. Un gate también se diseña para lo que no mira

Tres huecos de diseño, ninguno es un check roto.

- **Reglas cuya intersección no tiene salida.** Dos umbrales escritos por
  separado pueden no compartir ningún valor válido: uno exige menos de X, otro
  más de Y, con Y > X en cierto rango. Quien cae ahí recibe consejos opuestos y
  ningún mensaje le dice qué hacer de verdad. Al añadir una regla, **calcula
  su intersección con las vigentes**; si hay un rango imposible, no relajes una
  regla: dale su propio finding con la acción que sí sirve, y declara si eso
  cambia una política existente.
- **Gates que solo vigilan a quien se declaró.** Si el control salta solo
  cuando alguien registra una excepción, lo no declarado pasa libre y declarar
  sale caro. **Cubre también lo no declarado**; con que sea informativo basta,
  porque da nombre a un hueco invisible.
- **Señales que no separan baseline de deuda nueva.** Un sensor que cuenta el
  total de hallazgos está siempre en rojo en un repo con historia, o siempre en
  verde si se le sube el umbral hasta callarlo. Mide el **delta contra una
  línea base fijada**: lo que este cambio añade.

## Cuándo no aplica

Prototipos desechables y código sin consumidor aguas abajo: no merece un
backstop ni un `--force` registrado. Para un fallo que sí grita (excepción,
rojo, exit ≠ 0), basta lo normal.
