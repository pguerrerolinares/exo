# Sembrar una KB nueva con los proyectos del usuario

> **Para quién:** el agente que acaba de verificar una instalación
> ([`instalar-con-agente.md`](instalar-con-agente.md), paso 6). Necesitas la
> ruta de la KB y la carpeta donde el usuario tiene sus repos.

`exo init` deja la doctrina (`core/`, `learnings/`, `AGENTS.md`), pero
`projects/` queda vacío: el agente arrancaría sin saber en qué trabaja el
usuario. La siembra crea **una nota corta por proyecto activo**. No hace una
por cada repo, por dos razones:

- **El arranque tiene techo.** Cada proyecto ocupa una línea en
  `core-index`, y la plantilla deja sitio para unas 10.
- **La semilla es solo la ficha.** "Estado actual" y "Decisiones vivas" no
  salen de leer un README: las rellena `exo:document` al cerrar sesiones
  reales.

## 1. Candidatos, sin LLM

Repos con commits propios en los últimos 90 días, del más reciente al más
antiguo (≈10 s para 30 repos):

```bash
RAIZ="<carpeta-repos>"; YO="$(git config --global user.email)"; : "${YO:?sin user.email: no se pueden distinguir los commits propios}"
find "$RAIZ" -maxdepth 3 -type d -name .git -prune -print0 2>/dev/null |
while IFS= read -r -d '' g; do
  r="${g%/.git}"
  n=$(git -C "$r" rev-list --count --since=90.days --author="$YO" --all 2>/dev/null || echo 0)
  [ "$n" -gt 0 ] || continue
  printf '%s\t%s\t%s\n' "$(git -C "$r" log -1 --format=%cs --author="$YO" --all)" "$n" "$r"
done | sort -r
```

Columnas: último commit propio, nº de commits en 90 días, ruta. Si falta un
repo que el usuario sabe activo, puede que en él use otra identidad de git:
repite con `YO=<otro email>`.

## 2. El usuario elige

Enséñale la lista y propón agrupar: varios repos de un mismo frente son
**un** proyecto (p. ej. `NorlinePlus`, `NorlinePlusMVC` y `NorlinePlusFinnk`).
Descarta las PoCs cerradas y la propia KB. Máximo ~10 proyectos.

## 3. Una ficha por proyecto

Despacha un subagente por proyecto (`model: sonnet`), en tandas de 3-4. Cada
uno lee solo esto de sus repos:

- `README*`
- `CLAUDE.md` / `AGENTS.md`
- `pom.xml` o `package.json`
- `docs/decisions/`, si existe
- `git log -20 --format='%cs %s'`

Escribe un fichero temporal con este formato exacto: el frontmatter con
`tags` es obligatorio, porque sin él el pre-commit de la KB rechaza la nota.

```markdown
---
tags: [proyecto, <slug-del-proyecto>]
---

## Qué es
<1-3 frases: qué hace y para quién.>

## Stack
<lenguaje, framework y versiones, de pom.xml / package.json.>

## Cómo se trabaja
<build, test y arranque, de README / CLAUDE.md.>

## Repos y enlaces
<rutas locales de los repos y docs del repo que merece la pena abrir.>

## Estado actual
(Se rellena con `exo:document`.)
```

Reglas:

- Cada dato lleva su fuente (`fichero:línea`). Lo que no tiene fuente se omite.
- Enlaza la documentación del repo, no la copies.
- Nada de secretos: ni `.env`, ni propiedades con claves, ni tokens, ni URLs
  con credenciales.
- Que no pase de 2 KB.

Alta, con el `tier` explícito (sin él la nota nace sin tier y `exo budget`
falla):

```bash
exo write new --dir projects --tier stable --title "<Nombre del proyecto>" --from <fichero>
```

Si sale con 3 por candidatas duplicadas, enséñaselas al usuario. No uses
`--force` sin su visto bueno.

## 4. Índice y commit

En `<ruta-kb>/core/core-index.md`, sección «Proyectos activos», quita
«(Ninguno todavía…)» y añade una línea por proyecto:

```markdown
- [[<Título>|projects/<slug>]]: <qué es, ≤ 10 palabras>.
```

`<slug>` es la última parte del `permalink` del frontmatter de la nota
creada (p. ej. `permalink: kb-ana/projects/finnk-backend` → `finnk-backend`).
Luego:

```bash
exo index && exo budget && git -C "<ruta-kb>" add projects core/core-index.md && git -C "<ruta-kb>" commit -m "siembra: proyectos activos"
```

Si el pre-commit rechaza por presupuesto, acorta la línea o la nota. Nunca
subas un techo.

## 5. Cierre con el usuario

Cuéntale tres cosas:

- **Cómo crece la KB:** cerrando cada sesión con `exo:document`, que
  actualiza la ficha del proyecto y su bitácora, y con `exo:distill` de vez
  en cuando para consolidar.
- **La KB es suya y local, sin remote.** Puede llevar rutas, URLs internas y
  nombres de cliente: no la subas a GitHub. Si quiere copia de seguridad,
  que sea a un proyecto privado del GitLab interno.
- **Qué abrir para empezar:** `<ruta-kb>/core/core-index.md`.
