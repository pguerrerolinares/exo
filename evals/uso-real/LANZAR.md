# Lanzar la medición en uso real

Comandos en orden, desde la raíz del repo. `$EXO_KB` es el repo git de la KB; los transcripts viven en
`~/.claude/projects`; la cache (texto de mensajes, etiquetas) en `~/.cache/exo-uso-real`, nunca en el repo.

```bash
P=~/.claude/projects
C=~/.cache/exo-uso-real
R=evals/uso-real

# 1. Sellar (se niega si sello.json existe; --forzar solo si reselas a propósito)
python3 $R/sellar.py --proyectos $P --kb $EXO_KB --salida $R/sello.json

# 2. Etapa 1: detectar (tope 3 USD, exit 3 al alcanzarlo; reanuda desde la cache)
python3 $R/detectar.py --proyectos $P --cache $C --salida $R/detecciones.jsonl --sello $R/sello.json

# 3. Muestra 1 (80 ítems a ciegas en $C/etiquetar-etapa1.jsonl)
python3 $R/muestra.py --etapa 1 --proyectos $P --cache $C --sello $R/sello.json

# 4. [el dueño rellena "etiqueta": si|no en $C/etiquetar-etapa1.jsonl]

# 5. Validar etapa 1
python3 $R/validar.py --etapa 1 --etiquetas $C/etiquetar-etapa1.jsonl --detecciones $R/detecciones.jsonl --salida $R/validacion.json

# 6. Resultado
python3 $R/resultado.py --detecciones $R/detecciones.jsonl --validacion $R/validacion.json \
  --proyectos $P --sello $R/sello.json --salida $R/resultado.json
```

Si `resultado.json` trae `puerta.estado == "etapa2"`:

```bash
# 7. Etapa 2: atribuir (tope 25 USD)
python3 $R/atribuir.py --proyectos $P --kb $EXO_KB --detecciones $R/detecciones.jsonl --cache $C \
  --salida $R/atribuciones.jsonl --sello $R/sello.json

# 8. Muestra 2 (30 ítems en $C/etiquetar-etapa2.jsonl); el dueño rellena "atribucion_dueño"
python3 $R/muestra.py --etapa 2 --proyectos $P --cache $C --kb $EXO_KB --atribuciones $R/atribuciones.jsonl --sello $R/sello.json
python3 $R/validar.py --etapa 2 --etiquetas $C/etiquetar-etapa2.jsonl --atribuciones $R/atribuciones.jsonl --salida $R/validacion2.json

# 9. Resultado final
python3 $R/resultado.py --detecciones $R/detecciones.jsonl --atribuciones $R/atribuciones.jsonl \
  --validacion $R/validacion.json --validacion2 $R/validacion2.json \
  --proyectos $P --sello $R/sello.json --salida $R/resultado.json
```

`sellar.py` cierra la población con `--hasta` (por defecto, el instante de sellar en UTC): las sesiones que empiezan
después y los mensajes con `ts >= hasta` quedan fuera de todos los pasos que reciben `--sello`.
`puerta.estado == "sin_validar"` significa que `resultado.py` se corrió sin `--validacion`: el juez no está validado y la
puerta no estabiliza ni abre la etapa 2 hasta que haya `validacion.json` (paso 5).

## Ramas

- **`juez_invalido`** (kappa < 0,4): el juez no sirve y el resultado no estabiliza nada. Genera la muestra por regex
  para medir sin juez: `python3 $R/muestra.py --etapa regex --proyectos $P --cache $C --sello $R/sello.json`
  (escribe `$C/etiquetar-regex.jsonl`).
- **Menos de 30 correcciones** en la etapa 1: la población es corta. Re-sella incluyendo la reserva con un `--desde`
  anterior y repite desde el paso 2:
  `python3 $R/sellar.py --proyectos $P --kb $EXO_KB --salida $R/sello.json --desde <fecha anterior> --forzar`
  (`--forzar` sin `--hasta` conserva el `hasta` del sello previo: la reserva amplía la población hacia atrás, nunca
  hacia delante. Con `--sello`, `detectar.py`, `muestra.py` y `atribuir.py` toman `desde` del sello). Declara el cambio en el informe:
  el sello guarda el `desde` usado y `resultado.py` lo lee de ahí.

## Qué comprueba el sello

`detectar.py`, `atribuir.py` y `resultado.py` salen con 2 si el sello no casa: prompt, `claude_version`, `filtro_sha256`
y el modelo (y `lote` en detectar) de cada uno; `resultado.py` además rehace la población y compara `sesiones_sha256`.
`resultado.py` sale con 1 si una detección no está en la población, y marca `puerta.estado == "incompleto"` si faltan
detecciones de mensajes de la población (corrida a medias o fichero obsoleto). En la etapa 2, si los `sin_veredicto` son
más del 10 % de los positivos, `final.estado` es `incompleto`; se declaran en `n_sin_veredicto_etapa2`.
