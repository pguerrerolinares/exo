# C8 — runbook del cutover: lo que ejecuta Paul

> ## EJECUTADO el 2026-08-18. Estado: **cutover completo y verificado.**
>
> - **Fases 1, 2, 3, 4a, 4b, 5, 6: hechas.** El rename del repo lo ejecutó Paul;
>   el resto, el orquestador bajo `OVERRIDE` registrado en el ledger.
> - Verificación final en sesión fresca:
>   `PROCESS: brainstorm, debug, documenta, orchestrate, plan, tdd, verify` ·
>   `SUPERPOWERS: NINGUNA`.
> - **Fase 3 se ejecutó en orden INVERTIDO** respecto a lo escrito abajo: alta del
>   catálogo nuevo primero, installs, y baja del viejo al final. Los dos catálogos
>   tienen nombres distintos (`agent-develop` y `exo`), así que coexisten y no hay
>   ningún instante sin plugins resueltos. A cambio hay una ventana breve de
>   plugins duplicados, cerrada desactivando el trío viejo acto seguido.
> - **`git-subdir` contra un repo privado funciona**: `process@exo` bajó sus 7
>   skills desde `pguerrerolinares/exo.git`. Era el riesgo abierto nº3 del gate.
> - **Fase 7: hecha** en sesión nueva (2026-08-18, tarde). Paul borró
>   `cache/agent-develop`; con ella se fueron las huérfanas del plan (reflex
>   0.6.0/0.8.0/0.11.0/0.12.0 y paul-profile 0.2.1). `cache/exo` queda con una
>   sola versión por plugin. Las de `superpowers 6.x` (6.1.1/6.2.0/6.3.0) siguen
>   en pie: el rollback vive.
> - **Probe M3-02: verde.** Un `reflex:executor` real despachado post-cutover
>   emitió `type=reflex:executor perfil=reducido bytes=997` en
>   `~/.claude/reflex-log.jsonl`. El transporte de inyección sobrevivió al cambio
>   de marketplace — era el único fallo-sin-síntoma que quedaba abierto.
> - **Extra fuera de plan**: enterrada también la huérfana `understand-anything`
>   (plugin disabled desde marzo; 232 MB entre caché y marketplace, clon limpio de
>   `Lum1104/Understand-Anything`, reinstalable). Sus dos entradas salieron de
>   `~/.claude/settings.json`. `plugins/cache` y `plugins/marketplaces` quedan con
>   `claude-plugins-official` y `exo`, nada más.
>
> **A partir de aquí el rollback ya no es sin pérdida**: la caché vieja no existe,
> así que volver atrás exige reinstalar desde `pguerrerolinares/exo-plugins`. El
> rollback de skills (encender superpowers, apagar `process@exo`) sigue intacto.
>
> Lo que sigue queda como registro de lo planeado y como base del rollback.

> Todo lo que sigue es **acción externa o de entorno vivo**: pushes, rename del
> repo y escritura de `~/.claude/settings.json`. Ninguna la ejecuta la fábrica
> (config §Ejecución de gates, línea roja). El trabajo de repo ya está hecho y
> mergeado; esto es el gesto final.
>
> Diseño y razones: `docs/superpowers/specs/2026-08-18-m3-m1b-cutover-design.md`.
> Verdicts: `docs/superpowers/consultas/2026-08-18-c8/`.

**Duración estimada:** 15-20 min. **Ventana:** con todas las sesiones de Claude
Code cerradas — las Fases 3 y 4 van seguidas.

**Punto de no retorno único:** no crear nunca un repo nuevo llamado
`agent-develop`. Mataría los redirects de GitHub que sostienen el rollback.

---

## Fase 0 — Comprobación previa

```bash
git -C ~/Documentos/proyectos/exo status --short && git -C ~/Documentos/proyectos/exo log --oneline -1
git -C ~/Documentos/proyectos/agent-develop status --short && git -C ~/Documentos/proyectos/agent-develop log --oneline -1
git -C ~/Documentos/proyectos/kb-demo status --short && git -C ~/Documentos/proyectos/kb-demo log --oneline -1
```

Esperado: los tres árboles limpios, con el merge de C8 en cabeza.

## Fase 1 — Pushes

`process` se sirve por `git-subdir` desde GitHub: **sin este push, el install de
la Fase 3 no encuentra el plugin.**

```bash
git -C ~/Documentos/proyectos/exo push origin main
git -C ~/Documentos/proyectos/kb-demo push origin main
```

## Fase 2 — Rename del repo del catálogo

```bash
gh repo rename exo-plugins -R pguerrerolinares/agent-develop --yes
gh repo view pguerrerolinares/exo-plugins --json name,visibility
# Esperado: {"name":"exo-plugins","visibility":"PRIVATE"}

git -C ~/Documentos/proyectos/agent-develop remote set-url origin https://github.com/pguerrerolinares/exo-plugins.git
git -C ~/Documentos/proyectos/agent-develop push origin master
git -C ~/Documentos/proyectos/agent-develop remote -v
# Esperado: origin https://github.com/pguerrerolinares/exo-plugins.git (fetch y push)
```

El directorio local sigue llamándose `agent-develop` a propósito: hay rutas de
disco escritas en docs y en `fabrica/SKILL.md:11`. Renombrarlo compra estética y
rompe referencias.

## Fase 3 — Re-registro del marketplace (sesiones cerradas, seguido)

```bash
claude plugin marketplace remove agent-develop
claude plugin marketplace add pguerrerolinares/exo-plugins
claude plugin marketplace list
# Esperado: aparece "exo" con source pguerrerolinares/exo-plugins; agent-develop ya no está

claude plugin install process@exo        # primero el que sustituye
claude plugin install reflex@exo
claude plugin install paul-profile@exo
claude plugin install workflow-lint@exo
claude plugin list | grep "@exo"
# Esperado: los 4, enabled
```

Si `marketplace remove` se niega por plugins activos:
`claude plugin disable reflex@agent-develop paul-profile@agent-develop workflow-lint@agent-develop`
y repetir.

## Fase 4a — settings.json, con superpowers TODAVÍA encendido

Editar `~/.claude/settings.json`. En `enabledPlugins`, sustituir las tres keys
`*@agent-develop` por las cuatro `@exo`, **sin tocar aún la de superpowers**:

```json
"process@exo": true,
"reflex@exo": true,
"paul-profile@exo": true,
"workflow-lint@exo": true,
"superpowers@claude-plugins-official": true
```

En `extraKnownMarketplaces`, confirmar que la entrada vieja desapareció y la
nueva quedó con auto-update:

```json
"exo": {
  "source": { "source": "github", "repo": "pguerrerolinares/exo-plugins" },
  "autoUpdate": true
}
```

**Humo antes de apagar nada.** Arrancar `claude` en cualquier proyecto y verificar:

1. el bloque de recall de arranque aparece (reflex vivo);
2. las skills `process:*` están listadas — las 7;
3. no hay warning de `plugin-not-found`.

Este paso separado es lo que hace cumplible la línea roja «no desactives
superpowers sin que `process` esté listo»: si el humo falla, no se ha apagado nada.

## Fase 4b — Apagar superpowers

Solo con el humo verde:

```bash
claude plugin disable superpowers@claude-plugins-official
```

Reiniciar sesión. Probe de M3-02 — despachar un ejecutor real desde
`process:orchestrate` y comprobar que reflex sigue enchufado:

```bash
grep inject-emitted ~/.claude/reflex-log.jsonl | tail -1
# Esperado: type=reflex:executor perfil=reducido
```

Si ahí aparece `general-purpose` o un `model` explícito, reflex v2 se ha
desenchufado: es exactamente el fallo sin síntoma que M3-02 existe para evitar.

## Fase 5 — Retirar el cron residual

**Esta fase no depende de ninguna anterior, y conviene hacerla el mismo día del
merge, aunque el resto del cutover se posponga.** El cron apunta al *working
tree* del repo (`.../agent-develop/plugins/reflex/scripts/a1-freeze-watch.sh`),
no a la caché: en cuanto el merge aterriza en `master`, el fichero desaparece y
el cron de las 09:07 falla con ENOENT cada mañana hasta que se retire.

El cron hashea `orchestrate-personal`, que ya no existe. **No uses `crontab -r`**
— borra el crontab entero; el residuo es una sola línea:

```bash
crontab -l | grep -v "a1-freeze-watch" | crontab -
crontab -l | grep a1 ; echo "exit=$?"
# Esperado: sin líneas, exit=1
```

## Fase 6 — Verificación end-to-end

```bash
ls ~/.claude/plugins/cache/exo/
# Esperado: paul-profile  process  reflex  workflow-lint
git -C ~/.claude/plugins/marketplaces/exo remote -v
# Esperado: origin https://github.com/pguerrerolinares/exo-plugins.git
claude plugin list | grep -E "@exo|superpowers"
# Esperado: 4 @exo enabled; superpowers disabled
```

Y una sesión-fábrica de humo: `fabrica` debe resolver su motor sin `Unknown skill`.

## Fase 7 — Limpieza (solo tras Fase 6 verde) — **EJECUTADA**

```bash
rm -rf ~/.claude/plugins/cache/agent-develop
```

Entierra de paso las cachés huérfanas que arrastraba el plan (reflex
0.6.0/0.8.0/0.11.0/0.12.0 y paul-profile 0.2.1). Las de `superpowers 6.x`
**se conservan**: son el rollback.

---

## Rollback

La Fase 7 está ejecutada, así que el rollback **ya no es sin pérdida**: la caché
de `agent-develop` no existe y hay que reinstalar desde `pguerrerolinares/exo-plugins`
(el repo y sus redirects siguen en pie, y el punto de no retorno se respeta: nadie
ha creado un repo nuevo llamado `agent-develop`).

**Solo el cutover de skills** (el marketplace se queda como está):

```bash
claude plugin enable superpowers@claude-plugins-official
claude plugin disable process@exo
```

Reiniciar sesión. superpowers queda instalado-pero-apagado hasta que un ciclo
real de trabajo cierre sin carencias.

**Todo, incluido el rename:**

```bash
gh repo rename agent-develop -R pguerrerolinares/exo-plugins --yes
git -C ~/Documentos/proyectos/agent-develop revert --no-edit HEAD
git -C ~/Documentos/proyectos/agent-develop remote set-url origin https://github.com/pguerrerolinares/agent-develop.git
git -C ~/Documentos/proyectos/agent-develop push origin master
claude plugin marketplace remove exo
claude plugin marketplace add pguerrerolinares/agent-develop
claude plugin install reflex@agent-develop paul-profile@agent-develop workflow-lint@agent-develop
```

Y restaurar las keys `*@agent-develop` en `enabledPlugins`.
