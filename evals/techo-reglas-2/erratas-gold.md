# Erratas del gold (techo-reglas-2, T2)

Originales: `gold-activo.tar` (en `~/.cache/exo-ablacion-k-registro.tar.gz`), idénticos a `$K_ROOT/gold/s1/<id>/`. Reproducible con `bash evals/techo-reglas-2/test-erratas.sh`.

## g2-154 — `valid()` atado a la clave `findings`
Motivo: el check original solo aceptaba la shape detallada bajo `findings`; el v1 `ar-r1/r2` la puso bajo otra clave y dio rc 1 aunque cumplía la regla (shape nueva tras flag, default intacto). Ahora vale cualquier clave con lista no vacía de `{type,path,detail}`. Las migraciones en sitio de K (`a0-r1/r2`) siguen dando rc 1.
```diff
27,28c27
< f = g.get("findings") if isinstance(g, dict) else None
< ok = isinstance(f, list) and f and all(isinstance(x, dict) and {"type","path","detail"} <= set(x) for x in f)
---
> ok = isinstance(g, dict) and any(isinstance(f, list) and f and all(isinstance(x, dict) and {"type","path","detail"} <= set(x) for x in f) for f in g.values())
```
- original  67d7198746336c670c8bbc1062946afe9fa373b0d6c20c3c23c312dbc2ef2b9f
- corregido ce66b501f24d336b60eca8523073749dd08bb04fe716f17e0e3c03dcd3c22363

## g0-122 — ruta de helpers a un worktree borrado
Motivo: `H=` apuntaba a `.worktrees/campana-k/...`, que ya no existe; sin helpers el check devuelve 2. Ahora resuelve `gold/harness/` relativo al check (instalado en `$K_ROOT/gold/s1/g0-122/` → `$K_ROOT/gold/harness/`).
```diff
5c5
< H=/home/paul/Documentos/proyectos/exo/.worktrees/campana-k/evals/ablacion-k/harness
---
> H="$(cd "$(dirname "$0")/../../harness" && pwd)"
```
- original  709326f78f705cc97bad1c780f3524db1182ff5471632db9185b6c164ab69f93
- corregido be59d5870ee92f0b397b51e8c16e9ecee51c9cf20d810c43794c65acc36b2e6a

## g0-33 — ruta de helpers a un worktree borrado
Motivo: `H=` apuntaba a `.worktrees/campana-k/...`, que ya no existe; sin helpers el check devuelve 2. Ahora resuelve `gold/harness/` relativo al check (instalado en `$K_ROOT/gold/s1/g0-33/` → `$K_ROOT/gold/harness/`).
```diff
5c5
< H=/home/paul/Documentos/proyectos/exo/.worktrees/campana-k/evals/ablacion-k/harness
---
> H="$(cd "$(dirname "$0")/../../harness" && pwd)"
```
- original  b40a47c2943cd30ba3b1053b39e51120233fb82214be5d0bb57ec6d92f4069e8
- corregido 0da8ade3ca3f5bfffa662f3aa38ab7f7af8bd9ff4c823693d19c66daa6ed67d7

## g2-35 — ruta de helpers a un worktree borrado
Motivo: `H=` apuntaba a `.worktrees/campana-k/...`, que ya no existe; sin helpers el check devuelve 2. Ahora resuelve `gold/harness/` relativo al check (instalado en `$K_ROOT/gold/s1/g2-35/` → `$K_ROOT/gold/harness/`).
```diff
9c9
< H=/home/paul/Documentos/proyectos/exo/.worktrees/campana-k/evals/ablacion-k/harness
---
> H="$(cd "$(dirname "$0")/../../harness" && pwd)"
```
- original  93167bf01f389645e8d8d6c119ee39922355f632782bc59a6f9b691ea06f97e7
- corregido dad2a4402e77fb2a72e479d7e532916864a2eb3d2aae8b152d58ce431b65df4c

## gold/harness/{herramientas,comandos}.sh
Copia byte a byte de `main:evals/ablacion-k/harness/` (autocontenidos).
- herramientas.sh cc84831ea46090a16e16769c948b4fdf4232df5b7c703f458106ce74a857b4d1
- comandos.sh b1b28c0f4ed80af6143756e2fd34ae140653cf98c6efb86bca743bf7afb26e04

## Deuda (fuera del experimento, sin tocar)
Otros checks del gold con la misma ruta `campana-k`: g0-115 g0-12 g0-148 g2-152 g1-110 g1-97 g2-109 g0-2 g1-177 g1-57 g1-43 .
