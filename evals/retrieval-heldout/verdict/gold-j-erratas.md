# Erratas del gold J y de su pre-registro congelado

El pre-registro (`docs/superpowers/plans/2026-09-19-campana-j-preregistro-BORRADOR.md`)
quedó **congelado en `a936770`** el 2026-09-22 y su cabecera fija la regla:
*"Inmutable desde este commit; erratas → verdict."* Este fichero es ese verdict
de erratas. **Ninguna de ellas reabre el gold ni cambia su sha256**
(`5902ebc44b22447f609ce12ac0e3f015175e0786c8836ab48c54a9af3a6cf86a`). Las
recogieron el consultor-gate fresco que congeló el pre-registro y la review final
de la rama `a-juez-temperature`. La fase 2 tiene que leerlas antes de medir.

Sin texto de queries ni permalinks por fila: los ids `jNNN` son los del gold.

## E1 — El tope de gasto de la corrida válida no respetaba el STOP firmado

- **Qué dice el pre-registro** (§11): *"Coste acumulado > $10: STOP"*. El
  default del harness coincide: `TOPE_USD_DEFAULT = 10.0` (`juez.py`).
- **Qué se hizo**: la corrida válida se lanzó con `--tope-usd 20`. §10 lo
  recoge, pero **no se declaró como desviación**, a diferencia de F6
  (`temperature`) y F7 (`MAX_CHARS`).
- **Origen**: el dueño lo aprobó al elegir el tope de 12.000 caracteres, cuya
  opción decía literalmente *"Kimi ≈ $13,66; subo `--tope-usd` a 20"*
  (2026-09-20). La estimación de $13,66 superaba el STOP de $10, así que la
  corrida no habría cabido bajo él.
- **Si se violó en los hechos: no.** Coste acumulado contabilizado por el
  harness sumando **todas** las corridas, que es lo que pide la regla (§10 solo
  da la válida):

  | corrida | ok | desconocidas | USD |
  |---|---|---|---|
  | humo con `temperature 0` (HTTP 400) | 0 | 1 | 0,0000 |
  | humo con `temperature 1` | 3 | 0 | 0,0498 |
  | kit de 4.000, detenida por decisión del dueño | 17 | 1 | 0,3846 |
  | kit de 12.000, timeout de 120 s | 3 | 1 | 0,0725 |
  | **kit de 12.000, corrida válida** | 286 | 0 | **8,1045** |
  | **acumulado** | | **3** | **8,6114** |

  Más 6 llamadas de diagnóstico hechas fuera del harness para aislar el HTTP
  400 (3 rechazadas con 400, 3 aceptadas de ~280 tokens). Cada desconocida es
  una llamada interrumpida cuyo coste no llegó a registrarse; acotada por el
  coste de una llamada del kit grande (≈ $0,05), **el acumulado queda por
  debajo de $8,80**. El STOP de $10 no se alcanzó.
- **Nota**: la corrida válida registra 286 intentos `ok` para 285 filas: un
  intento recibió respuesta pero no pasó la validación y se reintentó. Estaba
  facturado y está contabilizado.
- **Lo que queda como errata**: el breaker estuvo **configurado** por encima de
  la regla firmada sin declararlo. Para la fase 2 rige el STOP de §11 (acumulado
  > $10); subirlo exige declararlo como desviación antes de lanzar.

## E2 — Dos etiquetas de `archive` son truncado residual (j141, j145)

En las dos, el hecho que responde **sí está** en una candidata, pero en los
caracteres **18.843** (j141) y **17.138** (j145) de su cuerpo, más allá del corte
de 12.000 de F7. Los dos jueces dijeron "no lo menciona", y acertaban sobre lo
que vieron. No es error de juicio: es la misma causa que motivó F7, en su versión
reducida (del 93 % de cuerpos truncados al 28 %).

**Consecuencia para la fase 2**: j141 perjudica, en 1 de las 8 filas de `archive`,
a un brazo que devuelva la nota viva en vez de la archivada. **La decisión D-C
se reporta con y sin j141.**

## E3 — El suelo de `archive` es real, y el desempate jugó en su contra

El suelo (≥ 8 no nulas) se cumple con **8 exactas**; la frase clave se verificó
fila a fila en las 8. El desempate *lenient* a favor de fable (declarado en §3)
descartó 2 filas en las que Kimi había elegido la rotación archivada. Con el
desempate inverso serían 10. **El suelo no se infló; si acaso, se subestimó.**

## E4 — Los 7 acuerdos *lenient* los ganó fable

En las 7 filas acordadas en modo *lenient* las dos direcciones eran válidas, y en
las 7 se quedó la de fable. Es neutro en *lenient*, que es el modo que decide
(D-J10); en *strict*, que es descriptivo, desplaza 7 filas.

## E5 — Redacción de la cabecera congelada

- Cita *"la regla de reapertura de esta misma cabecera"*, pero el commit de
  congelación sustituyó justo ese párrafo: la referencia solo se sostiene en el
  historial de git (`git show a936770^:<fichero>`, cabecera, última frase del
  párrafo *Estado*). El fondo es correcto: F6 (`3bcaa59`) y F7 (`a661e8d`) son
  del 2026-09-20, dos días anteriores al gold.
- Dice que F6 y F7 están *"ambas en §3"*; F7 toca también §8.
- Conserva *"Punto abierto (review, I-4): el T0 está pendiente"* seguido de la
  enmienda que lo cierra (*"cláusula no ejercida"*). Rastro histórico sin efecto:
  D-J8 quedó en 0,25 y §4 está intacto.

## E6 — Fecha del gate

§10 conserva el placeholder `GATE: PRE-REGISTRO J CONGELADO <fecha>`, inevitable
porque el gate es posterior a la congelación. **La línea real es
`GATE: PRE-REGISTRO J CONGELADO 2026-09-22`**, del consultor fable fresco.

## Pendientes de código (rama posterior, no bloquean el merge)

- **El test de `MAX_CHARS` no cubre el camino que llega al juez.** Fija el
  truncado en `nota()` (matado por mutación: sin `[:MAX_CHARS]` falla con
  `17018 != 12000`), pero con `paquete()` leyendo el fichero crudo en vez de
  `c["cuerpo"]` la suite sigue en verde. Añadir en `test_gold_j.py` una
  aserción sobre `paq["texto"]`, p. ej. `assertNotIn("x" * (jz.MAX_CHARS + 1), paq["texto"])`.
- **El comentario del timeout exagera**: *"esperar de más cuesta segundos"*,
  pero una conexión colgada cuesta hasta 600 s por intento (≈ 30 min por
  paquete con 3 reintentos y tolerancia > 0, con el lock tomado), y el
  `timeout` de `urllib` es por operación de socket, no total.
- **Preexistente**: `kimi()` solo captura `URLError`, `TimeoutError` y
  `JSONDecodeError`. Un corte de conexión (`RemoteDisconnected`,
  `ConnectionResetError`, que son `OSError`) revienta el proceso en vez de
  pararlo limpio. El resultado sigue siendo conservador, porque la marca
  `en_vuelo` ya sincronizada cuenta como desconocida al reanudar.
- **La suite Python del harness no corre en CI** (no aparece en `ci.yml`): su
  verde depende de correrla a mano.
