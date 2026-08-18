# Pre-registro del gate de `targets` (M6-04)

> Escrito ANTES de compilar el binario nuevo y ANTES de ver ningún output suyo.
> Spec §5.3. Si esta nota se edita después de la primera corrida, el gate deja
> de ser un pre-registro y hay que declararlo aquí.

## Por qué no hay paridad de ranking

`targets` no puede dar el mismo orden en los dos índices (spec §5.2): las
columnas FTS difieren (`title, content_stems` con stemming del pipeline Python
de basic-memory, `content_snippet`, `permalink` indexado y `prefix='1,2,3,4'`
contra `titulo, cuerpo` crudos en exo) y la multiplicidad de filas también (160
filas `type='entity'` para 143 entities en basic-memory; 1:1 en exo). El
tokenizer NO difiere: ambos usan `unicode61 tokenchars 0x2F`.

## Los topics

Cinco, elegidos para cubrir un término técnico, un nombre propio, una palabra
de dominio, un acrónimo y una consulta multipalabra:

1. `indexer`
2. `reflex`
3. `memoria`
4. `kbx`
5. `recall en el punto de uso`

## Criterio (por topic)

Se compara `kbx targets <topic> --json --limit 5` corrido con el **binario
viejo sobre `~/.basic-memory/memory.db`** contra el **binario nuevo sobre una
copia RO de `~/.exo/index.db`**, con el filtro `tipo='note'` todavía puesto.

- **PASA** si ≥3 de los 5 permalinks del top-5 de basic-memory aparecen en el
  top-5 de exo.
- Cada permalink ausente se explica por stemming, por multiplicidad de filas o
  por `size`/`tier` rancio en basic-memory (spec §7). **Una ausencia sin
  explicación es un FALLO del port**, no una diferencia aceptable.
- El gate global de `targets` pasa si **pasan 4 de los 5 topics**. Con 3 o menos,
  el port se investiga antes de seguir.

## Qué NO mide este gate

No mide orden, ni score, ni `size_bytes` (será distinto por diseño: `entity.size`
está rancio en 17 de 138 notas y el valor nuevo sale de `stat`), ni `snippet`
(los cuerpos indexados son distintos).
