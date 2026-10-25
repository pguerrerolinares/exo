# Verdict: techo-reglas-3b (g0-33 con 3, 5 y 10 globales)

Pre-registro sellado en `33ae6ba`. Las 9 corridas acabaron (`completed`), sin breakers, por 0,52 USD. Registro: `~/.cache/exo-techo-reglas-3b-registro.tar.gz`.

| brazo | globales (total) | `check.rc` r1,r2,r3 | cumple | lanza `Agent` | cita la regla |
|---|---|---|---|---|---|
| `arp903` | 3 (13) | 1,1,1 | 0/3 | 0/3 | 0/3 |
| `arp905` | 5 (15) | 1,1,1 | 0/3 | 0/3 | 0/3 |
| `arp910` | 10 (20) | 1,1,1 | 0/3 | 0/3 | 0/3 |
| ref.: `arp1` de techo-reglas-3 | 1 (1) | 0,0 | 2/2 | 2/2 | 2/2 |

## Decisión (regla pre-registrada)

`arp905` y `arp903` dan los dos ≤ 1/3, así que **el número de globales no explica la caída**. La siembra arranca con **5 globales redactadas como mecánicas**, sin reglas de proceso del tipo «lanza un agente», y se abre un issue sobre la dilución de las reglas de proceso. `arp910` replica el 0/2 de `arp20` (0/3).

## Lectura

- g0-33 solo se cumple cuando es la única regla del system prompt. Con 12 acompañantes (3 globales más las 10 de proyecto) ya cae, igual que con 19: 0 de 13 corridas con compañía, frente a 2 de 2 sola.
- En el diseño, la lista de proyecto quedó fija en 10, así que no distingue «pocas globales» de «cualquier compañía». Lo que sí muestra es que reducir las globales a 3 no salva una regla de proceso mientras siga habiendo reglas de proyecto.
- Encaja con techo-reglas-3: las 9 golds mecánicas no diluyen con 20 reglas. Una regla que pide un cambio de conducta caro (delegar en un agente) sí se pierde, y lo que se pierde es el mecanismo, no la calidad: la nota final es correcta en todas las corridas.
- Es una sola tarea con k=3: indica una dirección, no un efecto general de las reglas de proceso.
