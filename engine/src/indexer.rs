use crate::abre_db;
use crate::aristas::{reindexa_aristas_de_nota, resuelve_destinos};
use crate::nota::parsea_nota;
use crate::schema::crea_schema;
use crate::trozos::trocea;
use crate::vectores;
use crate::walker::walk_kb;
use crate::con_embedder_de_proceso;
use crate::config_embeddings;
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::time::UNIX_EPOCH;

/// Recencia de `ruta_rel` (relativa a `kb`) según el último commit de git
/// que la tocó — JAMÁS mtime (§6.2 regla 2: ningún `created:` en frontmatter,
/// un clone fresco resetea mtimes). `None` si el fichero no tiene commits
/// (nuevo, aún no versionado) o si `git` falla por cualquier motivo — la
/// columna `notas.git_epoch` admite NULL para este caso, no es un error de
/// indexado.
pub fn git_epoch_de(kb: &Path, ruta_rel: &Path) -> Option<i64> {
    let salida = Command::new("git")
        .arg("-C")
        .arg(kb)
        .arg("log")
        .arg("-1")
        .arg("--format=%ct")
        .arg("--")
        .arg(ruta_rel)
        .output()
        .ok()?;

    if !salida.status.success() {
        return None;
    }

    let texto = String::from_utf8(salida.stdout).ok()?;
    let texto = texto.trim();
    if texto.is_empty() {
        return None;
    }
    texto.parse::<i64>().ok()
}

/// Resultado de una corrida de `indexa` (base del `data` del envelope de
/// `index`/`rebuild`, spec §4).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Resumen {
    pub indexadas: usize,
    pub saltadas: usize,
    pub borradas: usize,
    /// Trozos que pasaron por el modelo en esta corrida (M6-01b). Campo
    /// aditivo del envelope: no sube `SCHEMA_VERSION` (envelope.rs).
    pub trozos_embebidos: usize,
    /// Trozos cuyo texto no cambió y reutilizaron su embedding almacenado.
    /// Es la métrica que dice si el cache está sirviendo de algo.
    pub trozos_reusados: usize,
}

/// Pipeline de `exo index`/`exo rebuild` (spec §3): walk de `kb` → por nota,
/// skip si `mtime` no cambió o reparse+reindex completo si cambió → borrado
/// de las notas cuya `ruta` ya no aparece en el walk. `git_epoch` se refresca
/// en cada reindex (regla 2: la recencia consumida por ranking/recall es
/// `git_epoch`, `mtime` es SOLO detección de cambio).
///
/// `aristas` se puebla desde los `[[wikilinks]]` del cuerpo de cada nota
/// (M2-04); `trozos`/`vectores` se pueblan aquí (M2-06): cada nota
/// (re)indexada se trocea (`trozos::trocea`, spec §2.1) y sus trozos se
/// embeben en batch, con `vectores.rowid = trozos.id` (§2). El embedder de
/// fastembed se inicializa perezosamente y una única vez por PROCESO
/// (`con_embedder_de_proceso`, no una variable local a esta función) para
/// que un `exo index` sin cambios no pague la carga del modelo (§3,
/// coherente con el skip por mtime).
pub fn indexa(kb: &Path, db_ruta: &Path) -> Result<Resumen> {
    let conn = abre_db(db_ruta)?;
    crea_schema(&conn)?;

    // Guarda de modelo (Ola 1 T1), primero de todo y ANTES de tocar
    // fastembed: aborta rápido y sin cargar el modelo si la config cambió
    // desde la última corrida. `cfg.modelo`/`cfg.dims` se reutilizan un poco
    // más abajo para el upsert de meta (junto a `kb_root`), así que se leen
    // una única vez aquí.
    let cfg = config_embeddings().context("leer config de embeddings")?;
    verifica_modelo(&conn, &cfg.modelo)?;

    // meta.kb_root: procedencia del índice (M6-04 §2.1). Se escribe en cada
    // corrida por upsert — si la KB se mueve, el índice siguiente lo refleja.
    // Canónica y absoluta: kbx la usa como raíz para abrir ficheros, y una
    // ruta relativa dependería del cwd del proceso que llame a kbx.
    let kb_abs = std::fs::canonicalize(kb)
        .with_context(|| format!("canonicalizar raíz de KB {}", kb.display()))?;
    conn.execute(
        "INSERT INTO meta (clave, valor) VALUES ('kb_root', ?1)
         ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
        params![kb_abs.to_string_lossy()],
    )
    .context("escribir meta.kb_root")?;

    // meta.modelo_embeddings/meta.dims_embeddings: se escriben aquí, junto a
    // kb_root, y NO al final de la función. La clave no describe "la última
    // indexación terminó bien": describe de qué modelo son los vectores que
    // hay en esta DB. Y como el bucle de abajo commitea por nota
    // (`tx.commit()`, transacción por nota), esa procedencia es un hecho
    // desde la PRIMERA nota commiteada, no desde que la corrida entera
    // termina. Si el upsert viviera al final y la corrida abortara a mitad
    // (panic, OOM, `kill -9`, disco lleno), las notas ya commiteadas
    // quedarían en disco con vectores del modelo actual mientras la clave
    // seguiría ausente o con el modelo previo; la corrida siguiente, con la
    // config ya cambiada a otro modelo, leería esa ausencia como "índice
    // viejo, migración silenciosa" (rama `None` de `verifica_modelo`) y
    // mezclaría vectores de ambos modelos bajo la propia guarda que existe
    // para impedirlo. Escribiendo la clave antes de embeber nada, un abort a
    // mitad deja `meta` ya apuntando al modelo de esta corrida, y la corrida
    // siguiente con otra config choca contra la guarda y aborta antes de
    // mezclar.
    conn.execute(
        "INSERT INTO meta (clave, valor) VALUES ('modelo_embeddings', ?1)
         ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
        params![cfg.modelo],
    )
    .context("escribir meta.modelo_embeddings")?;
    conn.execute(
        "INSERT INTO meta (clave, valor) VALUES ('dims_embeddings', ?1)
         ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
        params![cfg.dims.to_string()],
    )
    .context("escribir meta.dims_embeddings")?;

    let rutas_absolutas = walk_kb(kb)?;

    let existentes: HashMap<String, f64> = {
        let mut stmt = conn.prepare("SELECT ruta, mtime FROM notas")?;
        stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))?
            .collect::<rusqlite::Result<_>>()?
    };

    let mut indexadas = 0usize;
    let mut saltadas = 0usize;
    let mut trozos_embebidos = 0usize;
    let mut trozos_reusados = 0usize;
    let mut vistas: HashSet<String> = HashSet::new();

    for ruta_abs in &rutas_absolutas {
        let ruta_rel = ruta_relativa(kb, ruta_abs)?;
        vistas.insert(ruta_rel.clone());

        let mtime = mtime_de(ruta_abs)?;
        if existentes.get(&ruta_rel) == Some(&mtime) {
            saltadas += 1;
            continue;
        }

        let Some(n) = parsea_nota(ruta_abs)? else {
            eprintln!(
                "aviso: {} sin permalink en frontmatter, se salta (§6.2 regla 1)",
                ruta_abs.display()
            );
            continue;
        };

        let git_epoch = git_epoch_de(kb, Path::new(&ruta_rel));

        // Transacción por nota (M6, hallazgo del gate): sin esto, el upsert
        // de `notas` (mtime fresco) quedaba commiteado ANTES de embeber los
        // trozos. Un fallo a mitad (embed, lock, kill) dejaba la nota con
        // mtime nuevo y vectores viejos/ausentes, y la corrida siguiente la
        // saltaba para siempre (`existentes.get(&ruta_rel) == Some(&mtime)`
        // arriba). `unchecked_transaction` no exige `&mut Connection` (aquí
        // `conn` es inmutable) y hace rollback en `Drop` si no se commitea,
        // que es justo lo que queremos ante el `?` de cualquier paso
        // intermedio. `Transaction` derefa a `Connection`: las funciones que
        // reciben `&Connection` aceptan `&tx` sin cambios de firma.
        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO notas (permalink, ruta, titulo, tipo, mtime, git_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(permalink) DO UPDATE SET
               ruta = excluded.ruta, titulo = excluded.titulo, tipo = excluded.tipo,
               mtime = excluded.mtime, git_epoch = excluded.git_epoch",
            params![n.permalink, ruta_rel, n.titulo, n.tipo, mtime, git_epoch],
        )
        .with_context(|| format!("upsert de notas para {}", n.permalink))?;

        tx.execute(
            "DELETE FROM notas_fts WHERE permalink = ?1",
            params![n.permalink],
        )
        .with_context(|| format!("limpiar notas_fts previo de {}", n.permalink))?;
        tx.execute(
            "INSERT INTO notas_fts (titulo, cuerpo, permalink) VALUES (?1, ?2, ?3)",
            params![n.titulo, n.cuerpo, n.permalink],
        )
        .with_context(|| format!("insertar notas_fts de {}", n.permalink))?;

        reindexa_aristas_de_nota(&tx, &n.permalink, &n.cuerpo)
            .with_context(|| format!("reindexar aristas de {}", n.permalink))?;

        let (embebidos, reusados) = reindexa_trozos_de_nota(&tx, &n.permalink, &n.cuerpo)
            .with_context(|| format!("reindexar trozos/vectores de {}", n.permalink))?;
        trozos_embebidos += embebidos;
        trozos_reusados += reusados;

        tx.commit()
            .with_context(|| format!("commit de la nota {}", n.permalink))?;

        indexadas += 1;
    }

    let mut borradas = 0usize;
    for ruta_rel in existentes.keys().filter(|r| !vistas.contains(*r)) {
        let permalinks: Vec<String> = {
            let mut stmt = conn.prepare("SELECT permalink FROM notas WHERE ruta = ?1")?;
            stmt.query_map(params![ruta_rel], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?
        };
        for permalink in permalinks {
            conn.execute(
                "DELETE FROM notas_fts WHERE permalink = ?1",
                params![permalink],
            )?;
            conn.execute("DELETE FROM aristas WHERE origen = ?1", params![permalink])?;
            borra_trozos_y_vectores_de_nota(&conn, &permalink)
                .with_context(|| format!("borrar trozos/vectores de {permalink}"))?;
            conn.execute("DELETE FROM notas WHERE permalink = ?1", params![permalink])?;
            borradas += 1;
        }
    }

    // Pase final sobre TODA la tabla `aristas` (§diseño M2-04 punto 2): barato
    // sobre las ~115 notas de la KB, y hace que un link roto se cure solo en
    // cuanto la nota destino aparezca en un index/rebuild posterior.
    resuelve_destinos(&conn).context("resolver destino_permalink de aristas")?;

    Ok(Resumen {
        indexadas,
        saltadas,
        borradas,
        trozos_embebidos,
        trozos_reusados,
    })
}

/// Borra los `trozos` de `permalink` y sus `vectores` correspondientes
/// (`vectores.rowid = trozos.id`, §2). Usado tanto al reindexar una nota
/// cambiada (reparse ⇒ reindex completo, spec §3 paso 2) como al borrar una
/// nota cuya `ruta` ya no aparece en el walk (deferred del gate m2-03 que
/// se ejecuta aquí: la cascada de m2-03 solo llegaba a `trozos`).
fn borra_trozos_y_vectores_de_nota(conn: &Connection, permalink: &str) -> Result<()> {
    let ids: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM trozos WHERE permalink = ?1")?;
        stmt.query_map(params![permalink], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for id in ids {
        vectores::borra(conn, id).with_context(|| format!("borrar vector rowid={id}"))?;
    }
    conn.execute("DELETE FROM trozos WHERE permalink = ?1", params![permalink])
        .with_context(|| format!("borrar trozos de {permalink}"))?;
    Ok(())
}

/// Reindexa `trozos`+`vectores` de una nota: borra lo previo (idempotente
/// ante re-ejecución), trocea `cuerpo` (spec §2.1) y embebe en batch. Nota
/// sin trozos (cuerpo vacío) jamás toca el embedder de proceso — ni lo
/// inicializa si aún no lo estaba (Task 2 del brief: "solo si hay algo que
/// embeber").
fn reindexa_trozos_de_nota(
    conn: &Connection,
    permalink: &str,
    cuerpo: &str,
) -> Result<(usize, usize)> {
    // Cache de embeddings por CONTENIDO del trozo (M6-01b): se leen los
    // vectores previos ANTES de borrarlos, indexados por su texto. Un trozo
    // cuyo texto no cambió reutiliza su vector en vez de volver a pasar por
    // el modelo — patrón estándar (LlamaIndex guarda un hash por nodo en su
    // docstore y solo re-procesa lo que cambió). Medido en esta máquina:
    // ~1 s de carga del runtime + ~0,25 s por trozo, así que editar una
    // línea de una nota de 9 trozos pasa de 3,4 s a ~1,3 s; y una nota
    // reindexada por un cambio de frontmatter no paga NADA.
    //
    // La clave es el texto exacto, no la posición: insertar un párrafo al
    // principio desplaza todos los `orden` pero conserva los textos, así que
    // el cache sigue acertando donde un cache por (permalink, orden) fallaría
    // entero.
    let previos = embeddings_por_texto(conn, permalink)?;

    borra_trozos_y_vectores_de_nota(conn, permalink)?;

    let textos = trocea(cuerpo);
    if textos.is_empty() {
        return Ok((0, 0));
    }

    let pendientes: Vec<String> = {
        let mut vistos = HashSet::new();
        textos
            .iter()
            .filter(|t| !previos.contains_key(*t))
            .filter(|t| vistos.insert((*t).clone())) // un texto repetido se embebe una vez
            .cloned()
            .collect()
    };

    let recien_embebidos: HashMap<String, Vec<f32>> = if pendientes.is_empty() {
        // Ni una llamada al embedder: ni siquiera se inicializa el modelo si
        // no lo estaba (la propiedad de m2-06 se conserva).
        HashMap::new()
    } else {
        let vectores_nuevos =
            con_embedder_de_proceso(|embedder| embedder.embebe_batch(&pendientes))
                .with_context(|| {
                    format!("embed batch de {} trozos de {permalink}", pendientes.len())
                })?;
        pendientes.iter().cloned().zip(vectores_nuevos).collect()
    };

    let mut embebidos = 0usize;
    let mut reusados = 0usize;

    for (orden, texto) in textos.iter().enumerate() {
        let vector = match previos.get(texto) {
            Some(v) => {
                reusados += 1;
                v
            }
            None => {
                embebidos += 1;
                recien_embebidos
                    .get(texto)
                    .with_context(|| format!("embedding ausente del trozo {orden} de {permalink}"))?
            }
        };
        conn.execute(
            "INSERT INTO trozos (permalink, orden, texto) VALUES (?1, ?2, ?3)",
            params![permalink, orden as i64, texto],
        )
        .with_context(|| format!("insertar trozo {orden} de {permalink}"))?;
        let id = conn.last_insert_rowid();
        vectores::inserta(conn, id, vector)
            .with_context(|| format!("insertar vector del trozo id={id}"))?;
    }

    // `embebidos` cuenta trozos escritos con vector nuevo; si un mismo texto
    // se repite en la nota, el modelo lo vio una vez pero aquí suma varias.
    // Se prefiere así: la métrica responde "cuántos trozos NO se pudieron
    // reutilizar", que es la que interesa.
    Ok((embebidos, reusados))
}

/// Embeddings de los trozos actuales de `permalink`, indexados por su texto.
/// Se llama ANTES de borrar. Un trozo sin vector legible (blob corrupto,
/// fila ausente) simplemente no entra en el mapa: se re-embeberá, que es el
/// comportamiento seguro.
fn embeddings_por_texto(conn: &Connection, permalink: &str) -> Result<HashMap<String, Vec<f32>>> {
    let filas: Vec<(i64, String)> = {
        let mut stmt = conn.prepare("SELECT id, texto FROM trozos WHERE permalink = ?1")?;
        stmt.query_map(params![permalink], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()
            .with_context(|| format!("leer trozos previos de {permalink}"))?
    };

    let mut mapa = HashMap::with_capacity(filas.len());
    for (id, texto) in filas {
        if let Some(v) = vectores::lee(conn, id)? {
            mapa.insert(texto, v);
        }
    }
    Ok(mapa)
}

/// Aborta si el índice se construyó con un modelo de embeddings distinto al
/// que la config activa pide ahora. Sin esto, dos modelos de la misma
/// dimensión (768d: el jina actual y `multilingual-e5-base`) producen blobs
/// indistinguibles para `vectores::lee` — su `BYTES_ESPERADOS` filtra por
/// longitud, no por procedencia — así que un `exo index` incremental tras
/// cambiar el modelo mezclaría vectores de ambos en la misma tabla sin una
/// sola queja. `exo rebuild` no pasa por aquí con estado mixto: borra la DB
/// entera antes de indexar (`main.rs::corre`, `borra_antes`), así que la
/// clave no existe y esta guarda no tiene nada que comparar.
fn verifica_modelo(conn: &Connection, modelo_actual: &str) -> Result<()> {
    let previo: Option<String> = conn
        .query_row(
            "SELECT valor FROM meta WHERE clave = 'modelo_embeddings'",
            [],
            |r| r.get(0),
        )
        .optional()
        .context("leer meta.modelo_embeddings")?;

    match previo {
        // Índice viejo, de antes de esta guarda: no hay nada que comparar
        // todavía. Se deja pasar y el upsert del principio de `indexa`
        // (junto al de `kb_root`) escribe la clave por primera vez —
        // migración silenciosa hacia delante.
        None => Ok(()),
        Some(v) if v == modelo_actual => Ok(()),
        Some(v) => bail!(
            "el índice se construyó con {v}, la config pide {modelo_actual}: corre 'exo rebuild'"
        ),
    }
}

fn ruta_relativa(kb: &Path, ruta_abs: &Path) -> Result<String> {
    Ok(ruta_abs
        .strip_prefix(kb)
        .with_context(|| format!("{} no está bajo la raíz {}", ruta_abs.display(), kb.display()))?
        .to_string_lossy()
        .into_owned())
}

fn mtime_de(ruta: &Path) -> Result<f64> {
    let meta =
        std::fs::metadata(ruta).with_context(|| format!("metadata de {}", ruta.display()))?;
    let modificado = meta
        .modified()
        .with_context(|| format!("mtime de {}", ruta.display()))?;
    let duracion = modificado
        .duration_since(UNIX_EPOCH)
        .context("mtime anterior a unix epoch")?;
    Ok(duracion.as_secs_f64())
}
