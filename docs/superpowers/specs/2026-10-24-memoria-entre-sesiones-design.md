# Memoria entre sesiones: ¿hay algo más eficiente que exo? — diseño (etapa 1)

Fecha: 2026-10-24. Estado: diseño aprobado por el mantenedor; spec pendiente de
su revisión. Consulta adversarial: `docs/superpowers/consultas/2026-10-24-memoria/consultor-fable.md`
(nota de un consultor fable, con la sección 6 «Tras el debate»).

## 1. Pregunta y definición operativa

Pregunta del mantenedor: **¿existe una forma más eficiente que exo de tener
memoria persistente entre sesiones de Claude Code?**

Definición operativa (aprobada):

> En la sesión N+k, el agente actúa como si supiera lo decidido en las
> sesiones ≤ N, sin que se lo vuelvas a contar, al menor coste.

«Coste» = tokens de entrada, latencia y mantenimiento del dueño.

La pregunta tiene dos etapas, y esta spec cubre solo la primera:

- **Etapa 1 (esta spec):** mantener, simplificar o abandonar exo frente a la
  alternativa nativa de Claude Code.
- **Etapa 2 (solo si exo sobrevive):** mejor canal y forma de memoria dentro
  de exo. Fuera de alcance aquí.

## 2. Lo que ya se sabe (no se re-mide)

- **Campaña K** (`evals/ablacion-k/verdict-etapa1.md`, 2026-09-30): A3 (exo)
  frente a A0 (sin memoria), 40 tareas S1 × 2 réplicas, sonnet.
  - Δ +0,0625, IC95 [−0,025, +0,1625] ⇒ `NO CONCLUYENTE`.
  - A0 ya resolvía 0,575.
  - S2 (neutras) ⇒ `DAÑO`, atribuido a `recall-inject`, ya retirado.
  - A3 costó +22 % de tokens_in y +29 % de USD.
  - Potencia recalculada: 0,56 para Δ = 0,10 con n = 40. Harían falta ~71
    tareas para detectar 0,10 y ~32 para 0,15.
- **techo-reglas-2** (`evals/techo-reglas-2/verdict.md`): la regla en el
  system prompt con framing (`arp`) da 6/10 en tareas-suelo, frente a 0/10 de
  A0. La regla por `SessionStart` sin framing (`ar`, medida en la v1) da 5/10
  con el check corregido. En las reglas mecánicas los dos canales dan 4/5; en
  las de principio o proceso, `arp` 2/5 y `ar` 0/5.
- **Retrieval offline** (2026-10-24): con el gold v2 por pooling (dos jueces,
  κ 0.797), hit@5 sin `prompt` está saturado: 80/91 en J, 59/70 en C. El
  estrato `agent-search` (queries reales de agentes) da 51/53. El acceso por
  búsqueda no es el cuello que se ve offline.
- **Prevalencia** (`evals/ablacion-k/erratas.md`, E4): en 484 prompts reales
  hay 6 correcciones del tipo «otra vez» (≈ 1,2 %) y 0 citas de un learning
  por título. Es una cota **inferior**: solo cuenta quejas explícitas, no los
  fallos que el dueño corrige sin decirlo.
- **Memoria nativa de Claude Code** (documentación oficial,
  code.claude.com/docs/en/memory.md, consultada el 2026-10-24):
  - CLAUDE.md por niveles, cargado entero al arrancar.
  - Auto memory en `~/.claude/projects/<p>/memory/`: `MEMORY.md` hace de
    índice y solo se cargan al arrancar sus primeras 200 líneas o 25 KB, lo
    que llegue antes. Los topic files se leen bajo demanda.
  - Configuración: `autoMemoryEnabled`, `autoMemoryDirectory` y
    `CLAUDE_CODE_DISABLE_AUTO_MEMORY`.

## 3. Enfoques considerados

| enfoque | qué compara | trade-off | decisión |
|---|---|---|---|
| **(a) Solo acceso** | todos los brazos con el mismo contenido (snapshot de la KB) convertido a su formato | justo y barato; no mide quién escribe mejor ni a qué coste para el dueño | base de la etapa 1 |
| (b) Sistema completo | se reproducen sesiones históricas con cada brazo, que acumula su propia memoria | mide la definición entera; cada brazo re-corre la historia, muy caro | descartado |
| **(c) = (a) + piloto de escritura** | (a) más ~10 sesiones reproducidas en 2 brazos | cubre lo que (a) no ve sin multiplicar el coste | **elegido por el mantenedor** |

Dentro de (c), el diseño de brazos pasó por un consultor y un debate:

- **Cinco brazos (A0, N, G, X, L) por no inferioridad frente a X: descartado.**
  Comparar contra X por no inferioridad no es válido si X no ha demostrado
  efecto sobre A0, y K dijo X ≈ A0. Es el problema de *assay sensitivity*
  (ICH E10; D'Agostino et al., 2003). Además:
  - L (core + stable en CLAUDE.md, ~180k tokens) multiplica ≈ 10 los tokens
    de entrada por corrida;
  - N con una línea por nota no cabe: 286 notas son > 200 líneas y ~35 KB.
- **L' (solo core en el system prompt): descartado.** Las reglas del pool
  salen de core + stable, y core es ≈ 15 % de ese material. Sin estratificar
  por tier, L' mediría la cobertura del core, no el acceso.
- **G (KB + grep sin índice): descartado.** Duplica a N, que también es índice
  más Read.
- **Elegido:** cuatro brazos con un **oráculo** como ancla positiva y sondas
  **cribadas**, no elegidas a mano.

## 4. Diseño de la etapa 1

### 4.1 Brazos

Todos con el mismo modelo (sonnet, versión fijada al congelar) y la misma
versión de Claude Code. Todos con `--setting-sources ""`, el CLAUDE.md del repo
de la tarea sin la sección de memoria, la KB de producción prohibida y un tope
de USD por corrida. Igual que K.

| brazo | qué carga | acceso al resto |
|---|---|---|
| **A0** sin memoria | nada de memoria | — |
| **O** oráculo (`ar`) | la regla literal de la sonda por `SessionStart`, sin framing | — |
| **X** exo | bloque de arranque real (`exo-recall.sh`) sobre el snapshot | `exo search` / `exo recall` sobre el snapshot |
| **N** nativa | `MEMORY.md` = una línea por nota core + stable (~82 líneas, ~8 KB medidos sobre la KB viva el 2026-10-24; se re-miden sobre el snapshot) | las notas como topic files, leídas bajo demanda |

- O es el **techo de acceso por el canal de X**. No es «recuperación
  perfecta» pura: el canal `SessionStart` sin framing deja un techo más bajo
  en las reglas de principio (ver §2). Se declara así.
- N con solo core + stable es una **decisión de curación** obligada por el
  límite de `MEMORY.md`. Las notas log y archive no tienen línea en el índice,
  pero siguen accesibles como ficheros.

### 4.2 Sondas

**Pool:**
- las reglas del pool de K (`pool/reglas-g*.jsonl`, privado);
- decisiones fechadas de la KB del snapshot.

Las sondas de K que el oráculo volteó en techo-reglas-2 entran directas al
pool, pero pasan la criba como el resto. Los checks que miran nombres de
comando se reescriben sobre efectos (lección de K, g0-115).

**Criba pre-registrada en dos pasos:**
1. A0 × 2 réplicas: se queda la sonda con 0/2.
2. O × 2 réplicas sobre las supervivientes: se queda la que da ≥ 1/2.

**Objetivo:** 40 sondas-suelo. **Si el pool no llega a 40, se para y se
escala; no se rellena.** Sesgo declarado: la criba sobre A0 induce regresión a
la media. No contamina el resultado principal, porque A0 se vuelve a medir
con réplicas nuevas.

**Estrato neutro:** 15 sondas sin nada relevante en la KB. Mide daño y
coste.

### 4.3 Corridas

- Principal: 40 sondas × {A0, O, X, N} × 3 réplicas = 480 corridas.
- Neutro: 15 × {A0, X, N} × 2 = 90 corridas.

### 4.4 Métricas

- **Éxito por sonda:** fracción de réplicas con `check.rc == 0`, en
  {0, ⅓, ⅔, 1}. rc 1 y rc 2 cuentan como fallo, igual que en K.
- **Primaria de conducta:** Δ emparejada por sonda. Bootstrap sobre sondas
  (10.000 réplicas, semilla fijada en el pre-registro) y test de signos
  exacto sobre las discordantes, como K.
- **Secundaria:** R = (brazo − A0) / (O − A0), la fracción del techo que
  recupera cada brazo.
- **Primaria de coste:**
  - Δ emparejada de tokens_in frente a A0 sobre las 55 sondas (suelo y
    neutras);
  - más la **sobrecarga fija por sesión**, calculada en bytes: el bloque de
    arranque de X (~5,9 KB en K) y `MEMORY.md` de N (~8 KB).
  - El neutro se reporta aparte.
- **Descriptivas:** USD, turnos, segundos y lecturas de notas por brazo.

### 4.5 Reglas de decisión (pre-registradas)

- **R1 — N no inferior a X:** límite inferior del IC95 de Δ(N − X) > −0,15.
  Si además la mediana de tokens_in de N es menor que la de X, **gana N**:
  exo se abandona como capa de acceso o se reduce a escritor de la KB.
- **R2 — X superior:** límite inferior del IC95 de Δ(X − N) > 0 **y**
  Δ ≥ 0,15. Entonces **exo sobrevive** y se abre la etapa 2.
- **R3 — daño en el neutro:** para X y para N, límite inferior del IC95 de
  Δ(brazo − A0) en el neutro > −0,15. Si no se cumple, se declara daño de ese
  brazo, con independencia de R1/R2.
- **En cualquier otro caso:** `NO CONCLUYENTE` y manda el coste.
- **Desempate con prevalencia:** prevalencia (≈ 1,2 %, cota inferior) ×
  Δ(brazo − A0) en las sondas, frente a la sobrecarga fija por sesión. Se
  reporta siempre; decide solo si R1–R3 no deciden.

**Por qué el margen es 0,15:** con 40 sondas y 3 réplicas el semiancho del IC
es ≈ 0,12–0,14. Un margen de 0,10 pediría 76–125 sondas, que no caben en el
presupuesto.

### 4.6 Condiciones de invalidación (se pre-registran tal cual)

1. **Si O recupera < 0,30 en las sondas cribadas** (menos de 12/40 sondas con
   ≥ 2/3 réplicas rc = 0): el cuello es el cumplimiento, no el acceso. La
   etapa 1 se cierra con «simplificar exo a entrega de reglas», sin comparar
   accesos.
2. **Si el límite inferior del IC95 de Δ(X − A0) es ≤ 0:** exo no tiene efecto
   de acceso demostrado y la decisión se toma solo por coste, sin test de no
   inferioridad.
3. **Integridad:** cualquier corrida con escritura en el snapshot de la KB o
   en la carpeta de memoria fuera de lo previsto se excluye y se reporta. Si
   pasan de 5 % de las corridas, el resultado no se adjudica.

### 4.7 Cambios en el harness de K (antes de correr)

`evals/ablacion-k/harness/correr.sh` se extiende con un `case` por brazo
nuevo (precedente: `arp`, `ars`). Cambios obligatorios:

1. `autoMemoryEnabled` por brazo. Hoy está fijado a `false` para todos
   (`correr.sh:81`). N lo activa, con `autoMemoryDirectory` apuntando a una
   copia fresca por corrida.
2. **Restaurar el snapshot de la KB entre corridas** (hallazgo I2 de K, sigue
   abierto): `git checkout -- . && git clean -fdq` sobre la KB del snapshot.
   Lo mismo con la carpeta de memoria de N.
3. `fugas.py`: hooks por brazo para N, y vigilar escrituras en el snapshot y
   en la carpeta de memoria.
4. **Canario de N:** antes de la criba, una corrida que demuestre que
   `MEMORY.md` se carga con la configuración aislada (`--setting-sources ""`
   y `--settings`). No está verificado ejecutándolo.
5. Volcar por corrida el bloque inyectado (`SessionStart` de X y O,
   `MEMORY.md` de N). K no pudo verificar 2 de 7 tareas discordantes por no
   tenerlo.

### 4.8 Piloto de escritura (descriptivo, no decide la etapa 1)

- **Material:** unas 10 sesiones históricas reproducidas en dos brazos:
  - exo, que escribe en la KB;
  - auto memory nativa, que escribe en su carpeta.
- **Mismo disparador en los dos:** al cerrar, el prompt «cierra la sesión y
  persiste lo decidido». Sin él, en `-p` la memoria nativa casi nunca escribe
  y se mediría el disparador, no el almacén.
- **Medidas por sesión:**
  1. ¿la decisión D de la sesión aparece en el artefacto? Binario, juez a
     ciegas;
  2. bytes escritos;
  3. minutos del dueño para dejarlo limpio;
  4. lectura de vuelta: una sesión N+1 con el acceso ganador resuelve una
     sonda derivada de D.
- Se declara de antemano que **no decide** la etapa 1. Sirve para dimensionar
  una campaña de escritura si hiciera falta.

## 5. Presupuesto

- Criba: unos 40 USD.
- Principal y neutro: unos 60 USD.
- Piloto de escritura: < 10 USD.
- **Total ≈ 100 USD.**

Si la criba revela que el pool no da 40 sondas, se para antes del principal.

## 6. Fuera de alcance

- La etapa 2: canal (system prompt frente a hook), forma de memoria y
  reranker.
- Brazos de terceros (mem0, basic-memory).
- Medir la prevalencia con un instrumento nuevo. Se usa la cota de E4.
- Cambios en el engine o en el plugin.

## 7. Riesgos declarados

- El oráculo `ar` infravalora el techo en las reglas de principio. R es
  secundaria por eso.
- N depende de un límite de la plataforma (200 líneas / 25 KB) que puede
  cambiar entre versiones de Claude Code. Se congela la versión.
- Las sondas se derivan de la KB del dueño: el resultado es sobre esta KB y
  este uso, no sobre memoria de agentes en general.
- El orquestador sabe qué brazo es exo. La adjudicación final la hace un
  adjudicador fresco desde los datos crudos, como en K.

## 8. Decisiones del builder check (2026-10-24)

Decisiones del orquestador, no del mantenedor. Se marcan así para que se
puedan revisar.

- **Snapshot y fugas.** Un solo snapshot de la KB, congelado a un commit antes
  de la criba. Las sondas se derivan de decisiones anteriores a ese commit. Un
  veto automático (patrón `pool/brief-veto.md`) descarta toda sonda cuya
  respuesta literal, o su check, esté en la KB o en el repo de la tarea. Las
  sondas las redacta un subagente con los briefs de `pool/`, sin ver los
  brazos.
- **Índice de N.** Generación mecánica: `- [título](ruta) — <description del
  frontmatter, o la primera frase del cuerpo>`, para las notas core + stable.
  Se congela antes de la criba y no se ajusta viendo resultados.
- **Tier de origen de las sondas.** El pool sale de todos los tiers. Que N no
  tenga en el índice las notas log y archive es una limitación real de la
  plataforma, y forma parte de lo que se mide. Δ(N − X) se reporta además
  estratificado por el tier de la nota de origen. La decisión principal usa el
  pool entero.
- **Qué ve N.** Su carpeta de memoria (una copia fresca por corrida) contiene
  las 286 notas del snapshot como topic files. El snapshot `$P/kb` le queda
  denegado, igual que a A0 y a O. X conserva `exo search` sobre el snapshot.
- **Plan B del canario.** Si `MEMORY.md` no carga con `--setting-sources ""`,
  se prueba `--setting-sources project` con `autoMemoryDirectory` y se repite
  el canario. Si tampoco carga, se para y se escala. N nunca se inyecta por
  system prompt: dejaría de ser nativa.
- **Réplicas.** La invalidación 4.6.1 y todas las métricas se calculan solo
  con las réplicas nuevas de la corrida principal, nunca con las de la criba
  (evita la maldición del ganador). El techo esperado de O está acotado por
  el 5/10 de `ar` en techo-reglas-2.
- **Topes.** Tope total de 130 USD y el breaker de `tanda.sh` (media > 2M
  tokens por corrida). Antes de la tanda principal se corre una tanda piloto
  de N de 5 sondas × 1 réplica para ver su cola de tokens.
- **Neutro.** Las 15 sondas neutras se regeneran con `pool/brief-s2.md` y
  `validar_s2.py`. La ausencia en la KB se comprueba por grep de los términos
  clave.
- **Congelación.** `congelar.sh` se extiende con la semilla, la versión de
  Claude Code y el modelo. Cada corrida registra `claude --version` y el
  modelo de `system.init`. Una discrepancia excluye la corrida.
