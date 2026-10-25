# Pre-registro — Campaña M: memoria entre sesiones, etapa 1

> **Estado: REGLAS CONGELADAS el 2026-10-24, antes de la primera corrida de
> la principal.** Spec: `docs/superpowers/specs/2026-10-24-memoria-entre-sesiones-design.md`.
> Plan y runbook: `docs/superpowers/plans/2026-10-24-memoria-entre-sesiones.md`.
> Donde este documento y el plan choquen, manda este documento; donde choque
> con la spec, manda la spec salvo las desviaciones declaradas en §5.

## 1. Pregunta

¿Hay una forma más eficiente que exo de tener memoria persistente entre
sesiones de Claude Code? Definición operativa: en la sesión N+k, el agente
actúa como si supiera lo decidido en las sesiones ≤ N, sin que se lo vuelvan
a contar, al menor coste.

## 2. Qué se ha observado antes de congelar, y qué no

- **Visto:** solo corridas de **A0** y **O** en la criba
  (`corridas-criba/`), el canario de N (1 corrida), y el piloto de N (5
  corridas `n` × 1 en `corridas-piloto/`, solo su cola de tokens_in; §6).
- **No visto:** ninguna corrida de **X** (`a2`) sobre sondas ni neutras;
  ninguna corrida de la principal; ninguna corrida de las neutras en ningún
  brazo.
- Ninguna réplica de la criba ni del piloto entra en el análisis.

## 3. Diseño congelado

- **Modelo:** `claude-sonnet-5-5`, Claude Code 2.1.310. `--max-budget-usd 10`
  y `--max-turns 40` por corrida. Paralelo 1.
- **Snapshot:** KB `kb_commit 80cba57878d37583be1f2654ecb7d96c51dad8ab`,
  índice exo 0.9.0 (cuda). Tarball de K sha256
  `116964a9ed565e1c6ac6a887c20de956eae5f2aec48cb317bf4092b2db57abd9`.
- **Brazos:** `a0` (A0, CLAUDE.md mínimo), `ar` (O, la regla por
  SessionStart), `a2` (X, exo), `n` (N, auto memory nativa con `MEMORY.md` de
  `prep/memoria/`, 68 líneas, 15.550 B). Todos llevan el CLAUDE.md mínimo.
- **Sondas-suelo (40):** cribadas A0 × 2 (0/2) y luego O × 2 (≥ 1/2). La criba
  dio 43; `criba.py principal` se queda 40 barajando con
  `random.Random(20261024)`:
  `g1-15 g2-27 g1-46 g0-101 g0-9 g1-82 g1-147 g0-66 g1-136 g1-152 g0-43 g1-21 g0-12 g1-140 g2-138 g0-147 g0-33 g1-57 g2-154 g1-102 g0-87 g1-157 g2-143 g0-67 g0-55 g0-96 g2-23 g1-170 g2-160 g1-32 g1-47 g2-48 g2-141 g0-78 g1-14 g1-110 g0-110 g0-44 g0-149 g2-186`.
  Tier de origen (`sondas.tsv`): 4 core, 36 stable.
- **Neutras (15):** `nm-01` … `nm-15`, nuevas, autocontenidas (`setup.sh`,
  sin red), con check funcional oculto y ausencia en la KB comprobada por
  grep (brief: `~/.cache/exo-campana-m/redaccion/brief-neutro-M.md`).
- **Corridas:** 40 × {a0, ar, a2, n} × 3 = 480; 15 × {a0, a2, n} × 2 = 90.
  Total 570, en tandas de 40 con el breaker de `tanda.sh`.
- **Ficheros (sha256):**
  - `orden-principal.txt` `261854e1dfa473058a5e3c4893fff395beef1f4afa9823bb41c1e4266ca02cfb`
  - `sondas.tsv` `af8b5d35d7eeb55231707cf6d94372852a34109920b73329ff4e1a3968c24ec4`
  - `neutras.txt` `fb92e13ca66d164690aa686066ac1c8f38c57b748fbd7a94d67c38c1f6739036`
  - `gold-activo.tar` `92fd5b703bfe28fab51b194d354609e1cc3e36b196362a207d2dd0087eab6d19` (160 s1 + 15 s2);
    memoria `ea87fc99deada23d`; el resto en `congelacion-principal.txt`
    (2026-10-24T23:45:52+02:00).

## 4. Métricas, reglas e invalidaciones (copiadas del plan, Global Constraints)

- Éxito por sonda: fracción de réplicas con `check.rc == 0` (rc 1 y rc 2
  cuentan como fallo).
- Bootstrap sobre sondas, 10.000 réplicas, `random.Random(20261024)`,
  percentiles empíricos. Test de signos exacto bilateral sobre las
  discordantes.
- Reglas, margen 0,15:
  - R1: límite inferior del IC95 de Δ(n − a2) > −0,15, y además la mediana
    de tokens_in de n < la de a2 ⇒ `GANA N`.
  - R2: límite inferior del IC95 de Δ(a2 − n) > 0 y Δ ≥ 0,15 ⇒
    `EXO SOBREVIVE`.
  - R3: en el neutro, límite inferior del IC95 de Δ(brazo − a0) > −0,15 para
    a2 y para n; si no se cumple ⇒ `DAÑO <brazo>`.
  - Si los límites inferiores de Δ(a2 − a0) y Δ(n − a0) son ambos ≤ 0 ⇒
    `NI X NI N DEMOSTRARON EFECTO` + `RECOMENDACIÓN: memoria mínima`.
  - Resto: `NO CONCLUYENTE`, y manda el coste (solo a2 frente a n). La
    etiqueta `DESEMPATE: <brazo>` (prevalencia ≈ 1,2 % × Δ frente a la
    sobrecarga fija por sesión) se reporta siempre y no reetiqueta el
    veredicto.
  - Falta de datos para una regla ⇒ `SIN DATOS <regla>`, nunca un veredicto
    por vacuidad.
- Invalidaciones:
  - (1) O recupera < 0,30 (menos de 12/40 sondas con ≥ 2/3 réplicas rc 0)
    ⇒ `CUELLO = CUMPLIMIENTO`.
  - (2) límite inferior del IC95 de Δ(a2 − a0) ≤ 0 ⇒ se decide solo por
    coste.
  - (3) > 5 % de corridas con escritura fuera de lo previsto ⇒ no se
    adjudica. En `n`, escribir en su carpeta de memoria es lo previsto
    (aviso, no fuga); en los demás brazos, y en el snapshot en todos, es
    fuga.
- «Sin memoria» no es un veredicto admisible: el suelo es el CLAUDE.md
  mínimo.
- Coste: Δ emparejada de tokens_in frente a A0 sobre las 55 sondas, más la
  sobrecarga fija por sesión en bytes; el neutro se reporta aparte.
- Tope: 130 USD en total entre criba, piloto y principal (gastados antes de
  la principal: §6). El breaker para si la media de tokens_in por corrida
  pasa de 2M; una parada se reporta y espera al mantenedor.
- Análisis: `python3 evals/campana-m/analizar.py ~/.cache/exo-campana-m
  ~/.cache/exo-campana-m/sondas.tsv ~/.cache/exo-campana-m/neutras.txt`,
  recomputado por un adjudicador fresco desde los datos crudos
  (`evals/campana-m/verdict.md`).

## 5. Desviaciones del plan, declaradas

1. **Neutro nuevo** en lugar de 15 S2 de K (Ruling Final de la orquestación,
   aprobado): los repos y venvs de S2 no existen en esta máquina.
2. **Criba por tandas** en lugar de «200 candidatas y luego criba»: se
   redactó y cribó en tandas de 60 reglas hasta pasar de 40 sondas-suelo
   (aprobado por el mantenedor). Candidatas: 40 heredadas de K + 51 + 51.
3. **Solo checks `script`** (errata E8 de K): las reglas que solo admiten
   rúbrica son no convertibles.
4. **Exclusiones por check flojo**, decididas antes de ver ninguna corrida
   de esas tareas: `g0-60`, `g2-84` (proxy por vocabulario), `g0-68` (regla
   escrita en el CLAUDE.md del repo; rc 2 si no discrimina). Se quedaron con
   aviso `g1-95`, `g2-121` y `g2-66` (heurística léxica). Registro:
   `~/.cache/exo-campana-m/redaccion/decisiones.txt`.

## 6. Criba y piloto (descriptivo)

| | candidatas | pasan A0 (0/2) | pasan O (≥ 1/2) |
|---|---|---|---|
| heredadas de K | 40 | 16 | 12 |
| tanda 1 | 51 | 23 | 17 |
| tanda 2 | 51 | 19 | 14 |
| **total** | 142 | 58 | **43** |

0 fugas y 0 KB sucias en la criba. Gasto de criba: 29,53 USD.

Piloto de N (5 sondas de la lista × `n` × 1, `corridas-piloto/`):

| sonda | tokens_in n | tokens_in a0 (criba r1, r2) | usd n |
|---|---|---|---|
| g1-15 | 109.199 | 58.339, 58.379 | 0,10 |
| g2-27 | 134.626 | 44.082, 21.366 | 0,19 |
| g1-46 | 77.615 | 55.269, 74.591 | 0,09 |
| g0-101 | 132.513 | 94.989, 115.881 | 0,10 |
| g0-9 | 104.572 | 95.856, 95.248 | 0,08 |

Media de `n`: 111.705 tokens_in, sin cola: ninguna corrida pasa de 135k, y el
breaker es 2M. 0 fugas, 0 avisos de memoria, 0 KB sucias. `MEMORY.md` llega
inyectado y sus 68 enlaces resuelven en la copia por corrida.

**Visto también, y declarado:** el rc de las 5 corridas piloto (`n` 0/5).
No cambia ninguna regla de §4, que estaban fijadas en el plan antes del
piloto; esas 5 corridas no entran en el análisis.

Gasto total antes de la principal: 30,09 USD de 130.

## 7. Enmienda 1 — brazo `a2s` (X + reflejo search-first), 2026-10-25

> **Congelada antes de la primera corrida de `a2s` sobre sondas.** Motivo: el
> verdict de la etapa 1 (`verdict.md` §5 y §7.3) señala como amenaza principal
> que X no era exo de producción: le faltaba el reflejo `search-first.sh`
> (`PreToolUse`, matcher `^(Agent|Task|Edit|Write|NotebookEdit)$`), y X buscó en
> 1/120 corridas. El veredicto de la principal se mantiene; esta enmienda solo
> decide si lo invalida a favor de exo.

- **Visto antes de congelar:** el resultado completo de la principal (todos los
  brazos, `verdict.md`) y 3 canarios `a2s` sobre neutras (`nm-01..03`, r1, en
  `corridas-canario-a2s/`, rc 0/0/0). Ninguna corrida `a2s` sobre sondas.
- **Brazo `a2s`:** `a2` exacto (bloque `exo-recall.sh` en `SessionStart`, exo
  sobre el snapshot) + `search-first.sh` del repo (idéntico al de la caché del
  plugin 1.18.5) en `PreToolUse` con el matcher de producción. Sentinel por
  corrida (`SEARCH_FIRST_SENTINEL_DIR=$O`).
- **Hallazgo de plataforma que obliga a una diferencia:** con
  `--no-session-persistence` el hook no puede leer `transcript_path` y calla
  (`search-first-skip motivo=transcript-ilegible`, canario `nm-01`). `a2s`
  corre **sin** ese flag; la sesión persistida se mueve a `$O/sesion.jsonl`, que
  es el único rastro del aviso que ve el modelo (`system-reminder` adjunto al
  primer Edit/Write/Agent). Ninguna otra diferencia con `a2`.
- **Corridas:** las 40 sondas-suelo × `a2s` × 3 = 120, serie, paralelo 1,
  tandas de 40 con el breaker de `tanda.sh`, orden barajado con
  `random.Random(20261025)`: `orden-a2s.txt` sha256
  `e8b5570c715e022bb30260e6960f6b7334c69bb31636f3e69ba48788139e0977`.
  Congelación: `congelacion-a2s.txt` (mismos snapshot, gold, modelo
  `claude-sonnet-5-5` y Claude Code 2.1.310 que la principal).
- **Comparación:** contra `a0` (y `a2` como contexto) de `corridas-principal/`,
  no intercalados. Desviación declarada: se corren ~10 h después, misma versión
  y modelo. Sin neutras: en la principal salieron al techo en todos los brazos
  y no tienen sensibilidad.
- **Medidas:** éxito por sonda como en §4; `busca` = la corrida hace un
  `tool_use` Bash con `exo search|targets|recall` (regex de `search-first.sh` +
  `recall`); bootstrap de Δ(a2s − a0) y Δ(a2s − a2) con la semilla de §4; test de
  signos; Δ de tokens_in emparejada frente a a0 (media y mediana).
- **Condición pre-registrada** (`verdict.md` §7.3):
  - busca en ≥ 50 % de las corridas **y** límite inferior del IC95 de
    Δ(a2s − a0) > 0 ⇒ `EXO CON SEARCH-FIRST DEMOSTRÓ EFECTO` ·
    `RECOMENDACIÓN 1 INVALIDADA`;
  - busca ≥ 50 % y límite inferior ≤ 0 ⇒ `BUSCA Y NO APLICA` · `MEMORIA MÍNIMA
    CONFIRMADA`;
  - busca < 50 % ⇒ `EL REFLEJO NO HACE BUSCAR` (o `EFECTO SIN BÚSQUEDA` si el
    límite inferior > 0) · `MEMORIA MÍNIMA CONFIRMADA`;
  - > 5 % de corridas con escritura fuera de lo previsto ⇒ `NO ADJUDICABLE`.
- **Análisis:** `python3 evals/campana-m/analizar_a2s.py ~/.cache/exo-campana-m
  ~/.cache/exo-campana-m/sondas.tsv`, escrito y testeado antes de correr.
- **Tope:** 130 USD de campaña; gastados antes de la enmienda 82,02 USD.
