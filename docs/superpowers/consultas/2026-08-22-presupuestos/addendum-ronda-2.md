# Addendum — segunda ronda: la objeción de Paul a la propuesta ratificada

Escrito por el orquestador tras entregarle a Paul la síntesis. **La rechazó**, con
una frase corta y un dato que ninguno de los cuatro informes había normalizado.
Todo lo de aquí está medido por mí contra el repo de la KB; los scripts, en mi
scratchpad. Verificadlo si os parece frágil: ya me equivoqué una vez en esta
auditoría (la premisa del arranque) y no quiero repetirlo.

## 1. La objeción de Paul, literal

> "pero consolida ya lo hemos realizado varias veces y siempre muerde"

Y después, cuando le presenté las primeras cifras:

> "pero mueve poco, porque he estado de vacaciones y no he usado claude code
> durante casi un mes"

La primera ataca el corazón de la propuesta. La segunda me corrigió a mí.

## 2. El bucle, con las cuatro podas fechadas

`core/doctrina-agentes.md`, serie completa reconstruida desde git:

| poda | queda en | recuperación |
|---|---|---|
| 9-jul | 10.772 → 7.469 | 9.282 en **2 días** |
| 11-jul | → 8.493 | 11.965, y sigue hasta 45.788 el 2-ago |
| 3-ago | 45.788 → **24.131** (−21.657) | **28.703 al día siguiente** — por encima del techo sellado |
| 22-ago (hoy) | 26.999 → 19.783 (`/consolida`, mañana) | **19.967 la misma tarde** (`/documenta`) |

Cuatro podas, cuatro recuperaciones inmediatas. La del 3-ago es la que decide:
se retiraron 21 KB y en **un día activo** se habían repuesto 4,5.

**Esto es lo que Paul quiere decir con "siempre muerde", y tiene razón.**
`/consolida` no estabiliza: desahoga. Y el punto 2 de la Fase 1 (nombrar la
evicción editorial) mejora la *calidad* de la poda, pero nada de lo ratificado
cambia la *tasa de reposición*.

## 3. La corrección de Paul, aplicada: el ratio del abogado se reduce a la mitad

Hubo un mes casi sin actividad (hueco medido de 13 días del 4 al 17 de agosto, y
la semana 33 entera a cero; **40 días activos en total desde junio**). Al
normalizar por día activo en vez de por calendario:

| | por día calendario | **por día activo** |
|---|---|---|
| sin muro (11-jul → 2-ago) | 1.695 B | **3.108 B** (12 días activos) |
| en el muro (3-ago → 19-ago) | 10 B | **40 B** (4 días activos) |

El factor de contención del muro pasa de **180× a 78×**. Sigue siendo grande y
el A/B del abogado sobrevive — pero su magnitud era menos de la mitad de lo
declarado, y la cautela que él mismo apuntó ("agosto tiene 5× menos commits")
resultó ser el doble de grande de lo que suponía.

**Corolario incómodo para todos**: la calma de agosto que sostiene el A/B es, en
buena parte, ausencia de uso. El sistema lleva un mes sin ser puesto a prueba.

## 4. Lo que la corrección NO toca, y que reencuadra el problema

Son proporciones y hechos fechados, no tasas, así que las vacaciones no los
mueven:

- **El canon total no crece.** 269.947 B el 3-jul → **268.560 B hoy**. Oscila
  (pico de 387 KB, poda a 261 KB) pero lleva siete semanas plano.
- **El crecimiento está concentrado hasta lo absurdo.** De 11-jul a hoy, el canon
  entero creció +40.960 B; **las seis notas que más crecen suman +40.883 B**. Las
  otras ~44 notas, entre todas: **+77 bytes**.
- De esas seis, cuatro crecen por razones sanas: dos son **notas nuevas**
  (`evidencia-y-divulgacion`, `foss-jam-kit`, desde cero) y dos son **proyectos
  activos** (`lighthouses-bot`, `agent-solve-it`). Las otras dos son
  `doctrina-agentes` (+8.671) y `pragmatismo-y-pivots` (+8.773).

## 5. Las tres preguntas de esta ronda

**P1 — ¿La propuesta ratificada resuelve el bucle de §2, o solo lo retrasa?**
Sed literales. Si la respuesta honesta es "lo retrasa unas semanas y mejora la
calidad de la poda", decidlo: es un resultado válido y Paul puede aceptarlo
sabiendo lo que compra. Lo que no vale es venderlo como solución.

**P2 — Con §4 sobre la mesa, ¿el problema es de volumen o de continente?**
Mi lectura, que quiero que ataquéis: el canon no crece, y casi todo el
movimiento cae en dos notas cuyo título admite cualquier contenido futuro. Si es
así, el punto 3 de la Fase 1 (el test del título, hoy accesorio y limitado "al
momento del split, sin campaña") debería ser **la acción principal**: partir
`doctrina-agentes` y `pragmatismo-y-pivots` por tema. `landscape` ya sostenía
que el 22% de waivers son títulos mal factorizados; `diagnostico` sostenía que 9
de 11 son notas coherentes sobre temas grandes. **Esa contradicción sigue sin
resolverse y ahora decide la propuesta entera.** Resolvedla mirando esas dos
notas en concreto, no en abstracto.

**P3 — El criterio de Fase 2 está roto y hay que reescribirlo.** Dice "si tras
dos pasadas de `/consolida` vuelve a morder", en tiempo de calendario. Con 40
días activos en tres meses, unas vacaciones lo dan por cumplido sin haber
probado nada. ¿En qué unidad se mide — días activos, número de `/documenta`,
commits a canon? Que sea observable sin métricas nuevas (régimen §0).

## 6. Nota de método

Que nadie ceda por cansancio ni por complacer a Paul, que está harto y lo ha
dicho dos veces. Si la propuesta ratificada sigue siendo lo correcto pese a §2,
defendedla: un "sí, muerde, y aun así esto es lo mejor que hay" bien argumentado
es una respuesta legítima. Y si alguien cambia de posición, que sea por el dato.

En esta auditoría los cuatro habéis corregido algo propio con evidencia en
contra, y yo he tenido que retirar una premisa y recalcular un ratio. Ese es el
listón.
