# Gate KB semilla (Task 6, ola-1c-b) — verdict del auditor independiente

- **Fecha**: 2026-08-27 · **Worktree**: `C:/proyectos/homework/exo-wt-b` · rama `ola-1c-b`
- **Objeto**: las 11 notas de `engine/kb-template/` (6 de Task 6 sin commitear: `core/core-index.md`, `core/doctrina.md`, 4 de `learnings/`; más el diff sin commitear de `README.md` y el test `engine/tests/plantilla_presupuesto.rs`, también de esta ola).
- **Auditor**: fresco; no participó en escribir nada de lo juzgado.

## Veredicto: APROBADO CON CAMBIOS

**Conteo**: 0 BLOQUEANTES · 1 MAYOR · 5 MENORES.

Cambios que condicionan la aprobación (lista exacta):

1. **Retitular `core/core-index.md`**: el H1 y el `title` («core-index — mapa
   de memoria y doctrina compacta») y la cabecera «## Contrato de memoria» son
   transcripción casi literal de la nota privada (ver M-1). Basta un retitulado
   que diga lo mismo con palabras propias (p. ej. «core-index — mapa y
   presupuesto de esta KB»; «## Qué hay y dónde»).
2. **Desambiguar los tres `_template`**: dar a cada uno un `title` de
   frontmatter distinto, igual a su H1 ya existente («Plantilla de learning»,
   «Plantilla de bitácora», «Plantilla de proyecto»). Una línea por fichero
   (ver m-2).
3. **Decidir por escrito el nombre `recon-first.md`**: o waiver explícito en el
   registro del gate («identificador de producto, como `core-index`, exento de
   la regla nombres-en-español») o rename. Hoy viola la constraint verbatim
   (ver m-3).
4. **Despersonalizar el doc-comment de `engine/tests/plantilla_presupuesto.rs`**:
   quitar las métricas y la fecha de la KB privada (ver m-4).

Los MENORES m-5 y m-6 son recomendaciones; no condicionan.

---

## Verificación primaria propia (qué corrí y qué leí — no acepté nada del brief sin reproducirlo)

| Afirmación del brief | Reproducción propia | Resultado |
|---|---|---|
| `cargo test --release --test plantilla_presupuesto` → 2 passed | Corrido en el worktree | **2 passed; 0 failed** ✓ |
| `core-index.md` = 3.750 B ≤ 5.222 | `wc -c` sobre las 11 notas | **3.750 B** ✓ (cap 6.144, aire 15% ⇒ 5.222) |
| `fugas-semilla.sh` → EXIT=1 (limpio) | Corrido; además leí el script: 16 patrones (`paul`, `wisdom`, `empresa-x`, `cliente-a`, `equipo-x`, `cliente-c`, `cliente-b`, `redmine`, `universidad`, `lighthouse`, `spark`, `cge`, `solve-it`, `openwisdom`, `basic-memory`, `20YY-MM`), y su convención es **0 = encontró algo** | **FUGAS_EXIT=1** ✓ (sin match) |
| Render + index: 11 notas, 0 saltadas; 11 aristas, 0 sin resolver | Copié la plantilla a un tempdir del scratchpad, sustituí `{{KB_NAME}}` por `auditoria-kb` con sed, y corrí `exo.exe index --db <db temporal> --kb <tempdir>` (jamás toqué `~/.exo/`) | `index: indexadas=11 saltadas=0` ✓; en la DB temporal: **11 aristas, 0 con `destino_permalink` NULL** ✓ |
| «Se convirtieron 12 referencias a wikilink» | `grep -rn '\[\[' kb-template/` → 12 ocurrencias; 11 aristas porque el `UNIQUE (origen, destino_texto)` de `aristas.rs` colapsa el link duplicado de `README.md` (líneas 31 y 63, mismo destino) | **12 links = 11 aristas, cuadra** ✓ |

Constraints verbatim, comprobadas una a una:

- Directorios en inglés (`core`, `projects`, `learnings`, `log`, `archive`, `archive/log`) ✓. Nombres de nota en español ✓ **salvo `recon-first.md`** (m-3; `core-index` está mandatado por la spec y hardcodeado en el recall del engine, así que lo trato como identificador de producto, no como nombre de nota).
- `{{KB_NAME}}` único placeholder: `grep -rn '{{' | grep -v KB_NAME` → vacío ✓.
- `semilla: true` en las 11 notas ✓ (grep por fichero, 1 hit en frontmatter cada una; los hits extra de `AGENTS.md` son la sección que lo documenta).
- `tier: stable` en `core/`, `projects/`, `learnings/` y raíz; `tier: log` solo en `log/_template.md` ✓ (grep `^tier:` sobre los 11).
- Sin personas, clientes, proyectos concretos ni fechas en la plantilla ✓ (barrido de 16 patrones + lectura íntegra de las 11 notas). La única fecha del lote de la Task 6 está en el **test**, no en la plantilla (m-4).

Para el análisis de fugas leí **enteras** las 11 notas de la semilla y las comparé con sus contrapartes privadas leídas también enteras: `kb-demo/core/core-index.md`, `kb-demo/core/doctrina-agentes.md`, `kb-demo/learnings/El cuello de botella es el brief, no el modelo.md`, `.../Fallo silencioso — el instrumento que no grita.md`, `.../El padre integra, no implementa — despacho de subagentes.md` (que contiene las secciones «Orquestador limpio» y «Recon-first»).

---

## Hallazgos

### MAYOR

**M-1 · El esqueleto de `core-index.md` está transcrito de la nota privada, aunque la prosa esté limpia.**
- `engine/kb-template/core/core-index.md:9` — H1 «core-index — mapa de memoria **y** doctrina compacta» vs. privado `kb-demo/core/core-index.md` H1 «core-index — mapa de memoria **+** doctrina compacta». Mismo texto, un carácter de diferencia. La frase no aparece en ninguna spec publicada del repo (grep sobre `docs/`, `plugins/`, `engine/`): su única fuente es la nota privada.
- `engine/kb-template/core/core-index.md:14` — cabecera «## Contrato de memoria», idéntica a la sección privada del mismo nombre y en la misma posición (primera sección tras el H1).
- Colateral: las seis «formas concretas» de `learnings/fallo-silencioso.md:14-38` reproducen la taxonomía de seis mecanismos del privado **en el mismo orden** (degradar con forma válida → check no falsable → contrato por prosa → exit 0 → composición → ausencia ≠ evidencia). Cada ítem es genérico y no arrastra ni una anécdota; lo señalo como huella de derivación, no como fuga.
- **Por qué MAYOR y no BLOQUEANTE**: no hay ni un dato privado en el texto — cero nombres, cero clientes, cero anécdotas, cero fechas (verificado contra los originales, que están llenos de ellas: `empirica`, `pguerrero-music`, Navidrome, cifras, fechas). El daño de publicar tal cual es bajo. Pero el clean-room (spec `2026-07-16-framework-unificado-design.md:153`: «se escribe desde cero mirando la instancia solo como referencia de forma») es exactamente el control que este gate custodia, y un título calcado carácter a carácter es «mismo texto», no «mismo principio»: prueba que la nota privada estuvo a la vista al escribir. La respuesta honesta a «¿escrito desde cero o despersonalizado?» es: **la prosa, desde cero; el esqueleto del core-index, calcado**. El fix cuesta dos líneas (cambio nº 1).
- Descargo verificado en la otra dirección: la «regla de los índices» («retirar entradas muertas, nunca comprimir las vivas», `core-index.md:44-50`, `AGENTS.md:107-113`) **sí** está en doctrina publicada del propio repo (`docs/superpowers/specs/2026-08-23-contrato-editorial-design.md:101,236`), así que ese eco es del producto, no de la KB privada. Lo mismo vale para el cap 6.144/15%/5.222 (G3 de la spec, aserto literal en el test) y para los nombres `orquestador-limpio`/`recon-first`, que coinciden con doctrina y skill ya publicados en el plugin `exo`.

### MENOR

**m-2 · Tres notas comparten `title: _template`** (`learnings/_template.md:5`, `log/_template.md:5`, `projects/_template.md:5`). Verificado en la DB temporal: `select titulo, count(*) from notas group by titulo having count(*)>1` → `_template × 3`. `aristas.rs:59-61` desempata por orden de permalink (con el render actual ganaría `projects/_template`). Hoy ninguna arista apunta a `_template` (verificado sobre las 11), así que nada rompe. **Mi juicio sobre la pregunta 5 del brief: es deuda, y se cierra antes de publicar** — no por el desempate en sí, sino porque (a) el único mecanismo de enlace de exo es resolución por título y la propia semilla enseña a enlazar por título, así que regalar al tercero tres títulos ambiguos de fábrica es sembrar el fallo silencioso que `fallo-silencioso.md` predica; (b) `exo search` devuelve tres filas indistinguibles tituladas `_template`; (c) el fix es una línea por fichero y de paso arregla la incoherencia menor de que el `title` del frontmatter no case con el H1. Ruido aceptable no es: es barato de cerrar y gratis de no arrastrar.

**m-3 · `learnings/recon-first.md` incumple la constraint verbatim «nombres de nota en español»** — es el único nombre de nota de contenido íntegramente en inglés. Hay una defensa razonable (alinea con el skill publicado `exo:recon-first`, igual que `core-index` alinea con el permalink que el recall del engine hardcodea), pero una constraint verbatim no se incumple por criterio implícito de un ejecutor: o waiver escrito en el registro del gate o rename (cambio nº 3).

**m-4 · El doc-comment de `engine/tests/plantilla_presupuesto.rs:4-8` publica métricas y fecha de la KB privada**: «la KB de la que sale la doctrina: su `core-index.md` mide **5.355 B**», «el bloque de arranque … 5.921 B …; medido el **2026-08-27**». No nombra a nadie, pero es meta-información de la instancia privada, con fecha, en un fichero que viaja con el repo publicado — y el barrido de fugas no lo ve porque solo barre `kb-template/`. La motivación del gate cabe en una frase sin cifras ajenas (cambio nº 4).

**m-5 · Redundancia dentro del fichero con cap**: en `core/core-index.md:57-68`, tres de las cuatro viñetas de learnings repiten como glosa (tras el «—») casi literalmente el título que ya va dentro del wikilink (p. ej. línea 57-59: el link dice «Un orquestador delega la lectura pesada y se queda solo con la conclusión» y la glosa repite «un orquestador delega la lectura pesada y se queda solo con la conclusión, no con el material crudo»). Son ~200 B duplicados en la nota cuyo presupuesto el propio repo protege con test. La de `fallo-silencioso` (línea 63) lo hace bien: la glosa añade, no repite. Igualar las otras tres a ese patrón. Sobre la pregunta 4 del brief: las 12 conversiones son correctas (todas resuelven — verificado en DB —, ninguna referencia navegable quedó sin convertir: los `projects/proyecto-x.md` de `AGENTS.md` son ejemplos hipotéticos y están bien como texto plano; ninguna conversión sobra) y el alias-ruta como texto visible funciona; el único ruido real es esta duplicación glosa/título.

**m-6 · Huecos de utilidad para un tercero** (pregunta 2 — en conjunto la respuesta es buena, esto es lo que falta):
  - `README.md:35-41` promete que el índice de búsqueda «se regenera solo» y que «no hace falta mantener ningún índice a mano», y luego `core-index.md:70-75` exige mantener un índice a mano. Son dos sentidos de «índice» (el SQLite vs. la nota-mapa) y el texto no los distingue en ningún sitio; un recién llegado puede leerlo como contradicción (pregunta 3: es la única fricción de coherencia que encontré; frontmatter documentado vs. real, tiers, y reparto core-index/doctrina —mapa vs. puerta, con solape solo en los enunciados de una línea— están bien).
  - `archive/` solo trae `archive/log/`; `AGENTS.md:33-35,49-52` dice que lo `stable` también puede retirarse a `archive/` pero el árbol no ofrece sitio ni dice qué `tier` lleva una nota archivada.
  - Nada en `README.md` dirige al recién llegado a `core-index.md` como primera lectura; solo `core-index` se autodeclara punto de entrada (`core-index.md:11`). El recall del engine lo sirve mecánicamente a los agentes, pero el humano que abre la carpeta no lo sabe.

---

## Respuestas directas a las seis preguntas

1. **Fugas invisibles al grep**: leí las 11 notas enteras contra los originales privados. **Cero anécdotas, cero decisiones de contexto ajeno, cero ejemplos delatores** — los originales están saturados de casos con nombre y cifra y ninguno cruzó. La prosa se lee escrita desde cero; el esqueleto del `core-index` (título + primera cabecera) no (M-1), y la taxonomía de `fallo-silencioso` conserva orden aunque no texto.
2. **Utilidad**: sí — carpetas, `tier`, `semilla: true` y la regla de routing con ejemplo están explicados sin conocimiento asumido relevante. Huecos puntuales en m-6.
3. **Coherencia**: sin contradicciones de fondo; frontmatter documentado = frontmatter real; `core-index` y `doctrina` no sobran (mapa vs. puerta). Única fricción: los dos sentidos de «índice» (m-6).
4. **Wikilinks**: 12/12 correctos, todos resuelven, ninguno falta ni sobra; el ruido está en las glosas duplicadas (m-5).
5. **`_template` × 3**: deuda; cerrarla antes de publicar, fix de tres líneas (m-2).
6. **Calidad como producto**: alta. Prosa directa, concreta, con voz consistente y frases que se quedan («la foto, no el vídeo»; «da tranquilidad sin dar garantía»; «un índice que lista todo no es un índice, es una copia de la carpeta»). No suena a relleno generado; da confianza en la herramienta.

*Verdict emitido sin tocar notas ni código; el único fichero escrito es este.*

---

## Resolución del gate (orquestador, 2026-08-27)

Verdict aceptado. Los cuatro cambios exigidos se aplican. Resolución del nº 3,
que el auditor deja explícitamente a decisión escrita:

**`learnings/recon-first.md` se queda con ese nombre. Waiver concedido.**

Motivo: es un **identificador de producto**, no prosa. Coincide con el skill
publicado `exo:recon-first` y con el nombre que la propia §G3 de la spec fija en
su lista de ficheros (`learnings/recon-first.md`). La constraint «nombres de nota
en español» separa contenido de identificadores — es la misma línea que D7 traza
para verbos de CLI y claves de config, y la que ya exime a `core-index`.
Renombrarlo rompería la correspondencia con el skill que el usuario invoca, que
es peor que la incoherencia idiomática que cierra.

Lo que el auditor tiene razón en exigir no es el rename, sino que **la excepción
esté escrita**: un ejecutor no puede saltarse una constraint verbatim por
criterio implícito. Queda escrita aquí.

Los MENORES m-5 y m-6 no condicionaban, pero se aplican igualmente: m-5 libera
~200 B en el único fichero con presupuesto sellado por test, y los tres huecos de
m-6 son de una a tres líneas cada uno.
