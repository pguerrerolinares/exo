# Espejo en GitLab interno

> **Para quién:** quien mantiene el espejo `gitlab.norbolsa.com/poc-ia/plantillas/exo`.
> GitHub es el origen; GitLab solo recibe. El job vive en [`.gitlab-ci.yml`](../.gitlab-ci.yml).

GitLab 18.0 Free no trae *pull mirroring*, así que un pipeline programado
trae `main` y los tags de GitHub y los empuja al propio proyecto. Solo avanza
en fast-forward: nunca hace force-push.

**Consecuencia:** un merge a `main` en GitHub llega al equipo en la siguiente
ejecución del schedule. Empujar al espejo es publicar: el bump de versión del
plugin y el tag tienen que estar en `main` antes de mergear lo que dependa de
ellos.

## Alta (una vez)

En el proyecto de GitLab:

1. **CI/CD activado**: *Settings → General → Visibility* → *CI/CD* on.
2. **Project access token**: *Settings → Access tokens* → *Add new token*.
   - Rol **Maintainer** (`main` está protegida; Developer no puede empujar).
   - Scope **`write_repository`**.
   - Caducidad máxima permitida; apunta la fecha para renovarlo.
3. **Variable**: *Settings → CI/CD → Variables* → `MIRROR_PUSH_TOKEN` = el token.
   - *Masked and hidden*, *Protected* (el schedule corre en `main`, que es
     protegida), sin *Expand variable reference*.
4. **Schedule**: *Build → Pipeline schedules* → *New schedule*.
   - Cron `0 * * * *`, zona `Europe/Madrid`, rama `main`.
5. **Primera ejecución**: botón *Run* del schedule. El log debe mostrar
   `GitHub main = <sha> <asunto>` y el push de `main` y los tags nuevos.

Fuera del schedule se puede lanzar a mano con *Build → Pipelines → Run
pipeline* sobre `main`. Los pushes no disparan pipeline (regla `workflow`).

## Si falla

| Síntoma en el log | Causa | Arreglo |
|---|---|---|
| `falta la variable MIRROR_PUSH_TOKEN` | Variable ausente, o protegida y el pipeline no corre en `main` | Revisar paso 3 y que el schedule apunte a `main` |
| `HTTP Basic: Access denied` / `403` | Token caducado o sin rol Maintainer | Regenerar el token (paso 2) y actualizar la variable |
| `! [rejected] main -> main (fetch first)` | Alguien empujó directo al espejo | Llevar ese commit a GitHub o descartarlo; el espejo no se pisa solo |
| `! [rejected] vX.Y.Z (already exists)` | Un tag se movió en GitHub | Borrar el tag en GitLab a mano y relanzar |
| `Could not resolve host: github.com` / timeout | El runner no sale a internet | Variable `HTTPS_PROXY` en CI/CD con el proxy corporativo |
| `SSL certificate problem` contra GitHub | Proxy con inspección TLS | Variable `GIT_SSL_CAINFO` (tipo *File*) con la CA corporativa (incluida la del GitLab: la variable manda sobre la del runner) |
