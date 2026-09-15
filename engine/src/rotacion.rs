//! `exo rotate` — divide una bitácora `tier: log` en un prefijo frío
//! (archivado) y una cola caliente (que se queda), portado de
//! `kbx/internal/rotate` (`fe46443`). El corte es posicional, no por
//! fecha: solo 34 de 60 headings de la bitácora más irregular llevan fecha
//! ISO, así que parsear fechas sería frágil justo donde más importa
//! (`rotate.go`, comentario de cabecera del paquete).

use anyhow::{Context, Result};
use serde::Serialize;
use std::borrow::Cow;
use std::path::Path;
use std::sync::LazyLock;

/// El resultado de partir un contenido en preámbulo + frío + caliente.
/// Invariante duro: `preambulo + frio + caliente` reconstruye el original
/// byte a byte — nada se pierde (kbx `rotate.Plan`).
#[derive(Debug, PartialEq, Eq)]
pub struct Plan<'a> {
    pub preambulo: &'a [u8],
    pub frio: &'a [u8],
    pub caliente: &'a [u8],
    pub entradas_frias: usize,
    pub entradas_calientes: usize,
}

/// Cada línea de `contenido` con su offset de inicio, sin el terminador
/// `\n`. Si `contenido` no acaba en `\n`, la última línea (parcial) se
/// incluye igual — mismo contrato que `splitLinesKeepOffsets` en el Go
/// original, del que dependen `offsets_de_entradas`, `bloque_frontmatter` y
/// `frontmatter_sin_terminar`: una sola función de partir en líneas para
/// las tres, en vez de tres copias que puedan derivar.
fn lineas_con_offset(contenido: &[u8]) -> Vec<(usize, &[u8])> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off <= contenido.len() {
        match contenido[off..].iter().position(|&b| b == b'\n') {
            None => {
                out.push((off, &contenido[off..]));
                break;
            }
            Some(n) => {
                out.push((off, &contenido[off..off + n]));
                off += n + 1;
            }
        }
    }
    out
}

/// Offset de cada heading `"## "` a inicio de línea, ignorando los que
/// caen dentro de una valla ``` de código.
fn offsets_de_entradas(contenido: &[u8]) -> Vec<usize> {
    let mut offs = Vec::new();
    let mut en_valla = false;
    for (off, linea) in lineas_con_offset(contenido) {
        if linea.starts_with(b"```") {
            en_valla = !en_valla;
            continue;
        }
        if !en_valla && linea.starts_with(b"## ") {
            offs.push(off);
        }
    }
    offs
}

/// Mantiene las entradas más nuevas cuyo tamaño total, junto al preámbulo,
/// entra en `presupuesto_caliente`. Al menos una entrada se queda siempre
/// caliente (kbx `rotate.Split`).
pub fn parte(contenido: &[u8], presupuesto_caliente: i64) -> Plan<'_> {
    let offs = offsets_de_entradas(contenido);
    let Some(&primera) = offs.first() else {
        return Plan {
            preambulo: contenido,
            frio: &contenido[contenido.len()..],
            caliente: &contenido[contenido.len()..],
            entradas_frias: 0,
            entradas_calientes: 0,
        };
    };
    let preambulo = &contenido[..primera];

    let mut corte = offs.len() - 1;
    for i in (0..offs.len() - 1).rev() {
        let tamano = (contenido.len() - offs[i]) as i64;
        if preambulo.len() as i64 + tamano > presupuesto_caliente {
            break;
        }
        corte = i;
    }

    Plan {
        preambulo,
        frio: &contenido[offs[0]..offs[corte]],
        caliente: &contenido[offs[corte]..],
        entradas_frias: corte,
        entradas_calientes: offs.len() - corte,
    }
}

/// Línea delimitadora de frontmatter: `---` seguida solo de espacios/tabs,
/// tolerando también un `\r` final — mismo fix que ya aplica
/// `frontmatter::es_delimitador` para el resto de exo (sin él, un checkout
/// CRLF hace que un cierre `---\r` nunca matchee y el fallback sintético se
/// dispare en silencio sobre frontmatter válido; es el "fallo silencioso
/// canónico del port" que `frontmatter.rs` ya documenta, y esta función
/// replica la misma regla en vez de reintroducir la más estrecha de kbx).
fn es_delimitador_frontmatter(linea: &[u8]) -> bool {
    let mut fin = linea.len();
    while fin > 0 && matches!(linea[fin - 1], b' ' | b'\t' | b'\r') {
        fin -= 1;
    }
    &linea[..fin] == b"---"
}

/// El bloque `---...---` inicial, o un bloque sintético `tier: log` si
/// `contenido` no empieza por un delimitador o nunca cierra.
fn bloque_frontmatter(contenido: &[u8]) -> Cow<'_, [u8]> {
    const SINTETICO: &[u8] = b"---\ntier: log\n---\n";
    let lineas = lineas_con_offset(contenido);
    let Some(&(_, primera)) = lineas.first() else {
        return Cow::Borrowed(SINTETICO);
    };
    if !es_delimitador_frontmatter(primera) {
        return Cow::Borrowed(SINTETICO);
    }
    for &(off, linea) in &lineas[1..] {
        if !es_delimitador_frontmatter(linea) {
            continue;
        }
        let mut fin = off + linea.len();
        if fin < contenido.len() && contenido[fin] == b'\n' {
            fin += 1;
        }
        return Cow::Owned(contenido[..fin].to_vec());
    }
    Cow::Borrowed(SINTETICO)
}

/// ¿Empieza `contenido` con un delimitador de frontmatter que nunca cierra?
/// Distinto de "sin frontmatter": `aplica` (Step C) rechaza este caso en vez
/// de archivar entradas frías bajo metadata sintética que borraría en
/// silencio el tier/tags/permalink real de la nota.
fn frontmatter_sin_terminar(contenido: &[u8]) -> bool {
    let lineas = lineas_con_offset(contenido);
    let Some(&(_, primera)) = lineas.first() else {
        return false;
    };
    if !es_delimitador_frontmatter(primera) {
        return false;
    }
    !lineas[1..]
        .iter()
        .any(|&(_, l)| es_delimitador_frontmatter(l))
}

fn offset_delimitador_cierre(fm: &[u8]) -> usize {
    let mut cierre = 0usize;
    for (off, linea) in lineas_con_offset(fm) {
        if es_delimitador_frontmatter(linea) {
            cierre = off;
        }
    }
    cierre
}

/// Reescribe `clave: valor` dentro de un bloque de frontmatter,
/// añadiéndola antes del delimitador de cierre si no existía.
fn reemplaza_clave(fm: &[u8], clave: &str, valor: &str) -> Vec<u8> {
    let patron = format!("(?m)^{}:.*$", regex::escape(clave));
    let re = regex::bytes::Regex::new(&patron).expect("patrón de clave válido");
    if re.is_match(fm) {
        return re
            .replace_all(fm, format!("{clave}: {valor}").as_bytes())
            .into_owned();
    }
    let cierre = offset_delimitador_cierre(fm);
    if cierre == 0 {
        return fm.to_vec();
    }
    let mut out = Vec::with_capacity(fm.len() + clave.len() + valor.len() + 4);
    out.extend_from_slice(&fm[..cierre]);
    out.extend_from_slice(format!("{clave}: {valor}\n").as_bytes());
    out.extend_from_slice(&fm[cierre..]);
    out
}

/// Envuelve `s` como escalar YAML de comilla simple: la única regla de
/// escape es doblar una comilla simple embebida, así que es seguro para
/// cualquier contenido — a diferencia del estilo "plain", donde `#`, `&`,
/// `*`, `:` o `[` cambian lo que la línea significa para un parser YAML. El
/// título/slug de una nota los escribe Paul libremente.
fn comilla_simple_yaml(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

static PATRON_FECHA_ISO: LazyLock<regex::bytes::Regex> = LazyLock::new(|| {
    // `[0-9]`, no `\d`: en `regex` (Unicode activo por default) `\d` casa
    // cualquier dígito Unicode, no solo ASCII — RE2 de Go es ASCII-only, así
    // que esto iguala el comportamiento de kbx en vez de aceptar dígitos
    // (p.ej. arábigo-índicos) que una fecha ISO real nunca lleva.
    regex::bytes::Regex::new(r"[0-9]{4}-[0-9]{2}-[0-9]{2}").expect("regex de fecha ISO")
});

/// Deriva el nombre del archivo de las fechas ISO del bloque frío. Sin
/// fechas, cae a un número de secuencia (caso real:
/// desarrollo-agentico-bitacora, 26 de 60 headings sin fecha).
/// `existentes` es cuántos archivos ya hay para este slug.
pub fn nombre_de_archivo(slug: &str, frio: &[u8], existentes: usize) -> String {
    let mut fechas: Vec<&[u8]> = PATRON_FECHA_ISO
        .find_iter(frio)
        .map(|m| m.as_bytes())
        .collect();
    if fechas.is_empty() {
        return format!("{slug}-parte-{:02}.md", existentes + 1);
    }
    fechas.sort_unstable();
    format!(
        "{slug}-{}_{}.md",
        String::from_utf8_lossy(fechas[0]),
        String::from_utf8_lossy(fechas[fechas.len() - 1])
    )
}

/// Construye la nota archivada: el frontmatter original con `title` y
/// `permalink` reescritos, seguido del bloque frío verbatim. Con
/// `archivo_previo` no vacío, escribe un enlace hacia atrás justo después
/// del heading de título — la nota viva solo guarda UN aviso, apuntando al
/// archivo más reciente (Step C), así que cada archivo lleva el enlace
/// hacia atrás y la cadena se recorre archivo a archivo.
pub fn construye_archivo(
    frontmatter_original: &[u8],
    frio: &[u8],
    titulo: &str,
    permalink: &str,
    archivo_previo: &str,
) -> Vec<u8> {
    let mut fm = bloque_frontmatter(frontmatter_original).into_owned();
    fm = reemplaza_clave(&fm, "title", &comilla_simple_yaml(titulo));
    fm = reemplaza_clave(&fm, "permalink", &comilla_simple_yaml(permalink));

    let mut salida = Vec::with_capacity(fm.len() + frio.len() + titulo.len() + 64);
    salida.extend_from_slice(&fm);
    salida.extend_from_slice(b"\n# ");
    salida.extend_from_slice(titulo.as_bytes());
    salida.extend_from_slice(b"\n\n");
    if !archivo_previo.is_empty() {
        salida.extend_from_slice("> Continúa el histórico anterior en [[".as_bytes());
        salida.extend_from_slice(archivo_previo.as_bytes());
        salida.extend_from_slice("]].\n\n".as_bytes());
    }
    salida.extend_from_slice(frio);
    salida
}

#[derive(Serialize)]
pub struct Resultado {
    #[serde(rename = "note")]
    pub nota: String,
    #[serde(rename = "archive", skip_serializing_if = "Option::is_none")]
    pub archivo: Option<String>,
    #[serde(rename = "moved_bytes")]
    pub bytes_movidos: usize,
    #[serde(rename = "cold_entries")]
    pub entradas_frias: usize,
    #[serde(rename = "rotated")]
    pub rotado: bool,
}

impl Resultado {
    fn vacio(nota: String) -> Self {
        Self {
            nota,
            archivo: None,
            bytes_movidos: 0,
            entradas_frias: 0,
            rotado: false,
        }
    }
}

fn aviso_de_archivo(titulo: &str, entradas_frias: usize, bytes_movidos: usize) -> String {
    format!(
        "> Histórico anterior archivado en [[{titulo}]] ({entradas_frias} entradas, {bytes_movidos} B).\n\n"
    )
}

static PATRON_AVISO: LazyLock<regex::bytes::Regex> = LazyLock::new(|| {
    // UTF-8 real, no bytes latin1-escapados: en `regex::bytes` con Unicode
    // activo (default), `\xc3\xb3` casa el codepoint U+00C3 U+00B3, no los
    // bytes de «ó» — con eso el patrón nunca casaba «Histórico» y
    // `quita_aviso_previo` era un no-op (verificado con regex 1.13.1).
    regex::bytes::Regex::new(
        r"> Histórico anterior archivado en \[\[(.*)\]\] \(\d+ entradas, \d+ B\)\.\n\n",
    )
    .expect("patrón de aviso de archivo")
});

/// Quita el primer aviso de archivo del preámbulo de `contenido`, si lo
/// hay, para que una cadena de rotaciones no acumule un aviso por rotación
/// (el preámbulo nunca rota — `parte` siempre lo deja entero caliente).
/// Devuelve el contenido sin el aviso y el título al que enlazaba.
fn quita_aviso_previo(contenido: &[u8]) -> (Vec<u8>, String, bool) {
    let offs = offsets_de_entradas(contenido);
    let (preambulo, resto): (&[u8], &[u8]) = match offs.first() {
        Some(&o) => (&contenido[..o], &contenido[o..]),
        None => (contenido, &[]),
    };
    let Some(m) = PATRON_AVISO.captures(preambulo) else {
        return (contenido.to_vec(), String::new(), false);
    };
    let total = m.get(0).unwrap();
    let previo = String::from_utf8_lossy(&m[1]).into_owned();
    let mut nuevo = Vec::with_capacity(contenido.len());
    nuevo.extend_from_slice(&preambulo[..total.start()]);
    nuevo.extend_from_slice(&preambulo[total.end()..]);
    nuevo.extend_from_slice(resto);
    (nuevo, previo, true)
}

fn nombres_de_archivo_existentes(
    kb_root: &Path,
    dir_archivo: &Path,
    slug: &str,
) -> Result<Vec<String>> {
    let ruta = kb_root.join(dir_archivo);
    let entradas = match std::fs::read_dir(&ruta) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("listar {}", ruta.display())),
    };
    let prefijo = format!("{slug}-");
    let mut nombres = Vec::new();
    for entrada in entradas {
        let entrada = entrada.context("leer entrada de archive/log")?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        if nombre.starts_with(&prefijo) && nombre.ends_with(".md") {
            nombres.push(nombre);
        }
    }
    Ok(nombres)
}

fn desambigua_nombre_de_archivo(nombre: String, existentes: &[String]) -> String {
    if !existentes.iter().any(|n| n == &nombre) {
        return nombre;
    }
    let base = nombre.strip_suffix(".md").unwrap_or(&nombre);
    let mut n = 2;
    loop {
        let candidato = format!("{base}-{n}.md");
        if !existentes.iter().any(|e| e == &candidato) {
            return candidato;
        }
        n += 1;
    }
}

/// Crea un temporal único en `dir(ruta)`, le escribe `datos` y lo fsyncea.
/// Devuelve su ruta ya publicable por `rename`/`hard_link`. `File::create`,
/// `write_all` y `sync_all` corren dentro de la misma clausura: si
/// cualquiera de los tres falla, el `if let Err` de abajo borra el
/// temporal antes de propagar el error. Para un fallo de `File::create`
/// ese `remove_file` es un no-op (nunca llegó a existir nada que limpiar),
/// no una rama aparte — es la misma limpieza para los tres casos, igual
/// que el `defer os.Remove(tmpName)` de kbx justo tras `CreateTemp`, que
/// limpia en cualquier salida (I1 de la review: sin esto, el `?` de una
/// escritura o fsync fallidos salía antes de borrar el `.tmp`, dejándolo
/// huérfano). Compartido por `escribe_fichero_atomico` y
/// `escribe_fichero_exclusivo` (I3 de la review): ambas solo difieren en
/// cómo publican el temporal ya escrito.
fn escribe_temporal(ruta: &Path, datos: &[u8]) -> Result<std::path::PathBuf> {
    use std::io::Write;
    let dir = ruta.parent().context("ruta sin directorio padre")?;
    let base = ruta
        .file_name()
        .and_then(|n| n.to_str())
        .context("nombre de fichero no UTF-8")?;
    let unico = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = dir.join(format!("{base}.{}.{unico}.tmp", std::process::id()));

    let escritura = (|| -> Result<()> {
        let mut f = std::fs::File::create(&tmp)
            .with_context(|| format!("crear temporal {}", tmp.display()))?;
        f.write_all(datos)
            .with_context(|| format!("escribir temporal {}", tmp.display()))?;
        f.sync_all()
            .with_context(|| format!("fsync temporal {}", tmp.display()))
    })();

    if let Err(e) = escritura {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(tmp)
}

/// Escribe `datos` en `ruta` atómicamente: temporal en el mismo directorio,
/// `fsync`, `rename`. Cualquier lector ve o el contenido completo viejo o
/// el completo nuevo, nunca uno truncado. Sin `tempfile`: el nombre único
/// sale de PID + tiempo, sin subir esa dependencia de dev a producción.
fn escribe_fichero_atomico(ruta: &Path, datos: &[u8]) -> Result<()> {
    let tmp = escribe_temporal(ruta, datos)?;
    let resultado = std::fs::rename(&tmp, ruta)
        .with_context(|| format!("renombrar {} a {}", tmp.display(), ruta.display()));
    if resultado.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    resultado
}

/// Como `escribe_fichero_atomico`, pero **nunca** pisa un fichero
/// existente (`hard_link` falla si el destino ya existe, atómicamente, sin
/// listar el directorio antes). Backstop para el archivo: aunque la
/// desambiguación de arriba tuviera un bug, esto solo puede fallar con un
/// error, nunca destruir historia ya archivada. La nota viva, que SÍ debe
/// sobrescribirse en cada rotación, sigue usando `escribe_fichero_atomico`.
fn escribe_fichero_exclusivo(ruta: &Path, datos: &[u8]) -> Result<()> {
    let tmp = escribe_temporal(ruta, datos)?;
    let resultado = std::fs::hard_link(&tmp, ruta).with_context(|| {
        format!(
            "crear {} (ya existe: no se sobrescribe un archivo)",
            ruta.display()
        )
    });
    let _ = std::fs::remove_file(&tmp);
    resultado
}

/// `fsync` del directorio, para que la entrada nueva sea durable antes de
/// tocar la nota viva (si el proceso muere entre las dos escrituras, la
/// duplicación es preferible a la pérdida). El `File::open` corre SIEMPRE,
/// también en Windows — es lo que hace que una ruta mala falle alto en vez
/// de saltarse en silencio, que es la propiedad de la que depende el
/// llamador (igual que kbx `fsyncDir`, que abre el directorio antes de
/// mirar el SO).
fn fsync_directorio(dir: &Path) -> Result<()> {
    if cfg!(windows) {
        // Windows no tiene fsync de directorio: `FlushFileBuffers` sobre un
        // handle de directorio devuelve ERROR_ACCESS_DENIED, así que
        // `sync_all` aquí fallaría en cada rotación. NTFS journala los
        // metadatos del rename por su cuenta, así que no hay nada que
        // flushear ni un equivalente portable que llamar en su lugar.
        // Tampoco se puede `File::open` un directorio en Windows (a
        // diferencia del `os.Open` de Go, std no pasa
        // FILE_FLAG_BACKUP_SEMANTICS): se comprueba que existe y es un
        // directorio, que es lo que el Open de kbx garantiza aquí — una ruta
        // mala sigue fallando alto.
        let meta = std::fs::metadata(dir)
            .with_context(|| format!("abrir directorio {}", dir.display()))?;
        anyhow::ensure!(meta.is_dir(), "{} no es un directorio", dir.display());
        return Ok(());
    }
    let f =
        std::fs::File::open(dir).with_context(|| format!("abrir directorio {}", dir.display()))?;
    f.sync_all()
        .with_context(|| format!("fsync de {}", dir.display()))
}

/// Rota una nota de log. Con `escribe=false` nada toca disco: el resultado
/// informa qué pasaría. Con `escribe=true` crea `archive/log/<nombre>` y
/// reescribe la nota como preámbulo + aviso de archivo + cola caliente.
/// Nada se borra jamás: cada byte del original queda en la nota o en el
/// archivo. `nombre_kb` es el prefijo del `permalink` que se escribe en el
/// archivo (Decisión D-4: `exo::nombre_kb()`, no un literal fijo).
pub fn aplica(
    kb_root: &Path,
    ruta_rel: &str,
    presupuesto_caliente: i64,
    escribe: bool,
    nombre_kb: &str,
) -> Result<Resultado> {
    let completa = kb_root.join(ruta_rel);
    let contenido =
        std::fs::read(&completa).with_context(|| format!("leer {}", completa.display()))?;

    if frontmatter_sin_terminar(&contenido) {
        anyhow::bail!(
            "no se puede rotar {ruta_rel:?}: el contenido empieza con un delimitador de \
             frontmatter \"---\" pero no se encontró el cierre; añade el delimitador de cierre"
        );
    }

    let (contenido, archivo_previo, _) = quita_aviso_previo(&contenido);

    let plan_inicial = parte(&contenido, presupuesto_caliente);
    if plan_inicial.entradas_frias == 0 {
        return Ok(Resultado::vacio(ruta_rel.to_string()));
    }

    let slug = Path::new(ruta_rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(ruta_rel)
        .to_string();
    let dir_archivo = Path::new("archive").join("log");
    let existentes = nombres_de_archivo_existentes(kb_root, &dir_archivo, &slug)
        .with_context(|| format!("listar el directorio de archivo para {ruta_rel:?}"))?;

    // Búsqueda de punto fijo: el aviso que se antepone a la cola caliente
    // consume presupuesto, así que hay que volver a partir contra lo que
    // REALMENTE queda tras reservarle sitio, y repetir hasta que el corte
    // se estabilice. `parte` solo puede mover MÁS entradas a frío al bajar
    // el presupuesto (nunca menos), así que `entradas_frias` no decrece de
    // una pasada a otra y el bucle converge en, como mucho,
    // `offsets_de_entradas(contenido).len() + 1` pasadas.
    let aviso_para = |p: &Plan| -> String {
        let nombre = nombre_de_archivo(&slug, p.frio, existentes.len());
        aviso_de_archivo(
            nombre.trim_end_matches(".md"),
            p.entradas_frias,
            p.frio.len(),
        )
    };
    let max_iter = offsets_de_entradas(&contenido).len() + 1;
    let mut plan = plan_inicial;
    let mut aviso = aviso_para(&plan);
    for i in 0.. {
        if i >= max_iter {
            anyhow::bail!(
                "error interno: la búsqueda de presupuesto de rotación para {ruta_rel:?} \
                 no convergió tras {max_iter} pasadas"
            );
        }
        let siguiente = parte(&contenido, presupuesto_caliente - aviso.len() as i64);
        let siguiente_aviso = aviso_para(&siguiente);
        let estable = siguiente.entradas_frias == plan.entradas_frias;
        plan = siguiente;
        aviso = siguiente_aviso;
        if estable {
            break;
        }
    }

    let nombre = desambigua_nombre_de_archivo(
        nombre_de_archivo(&slug, plan.frio, existentes.len()),
        &existentes,
    );
    let aviso = aviso_de_archivo(
        nombre.trim_end_matches(".md"),
        plan.entradas_frias,
        plan.frio.len(),
    );
    let archivo_rel = dir_archivo.join(&nombre);
    let archivo_rel_str = archivo_rel.to_string_lossy().replace('\\', "/");

    let mut resultado = Resultado {
        nota: ruta_rel.to_string(),
        archivo: Some(archivo_rel_str.clone()),
        bytes_movidos: plan.frio.len(),
        entradas_frias: plan.entradas_frias,
        rotado: true,
    };
    if !escribe {
        return Ok(resultado);
    }

    let titulo = nombre.trim_end_matches(".md").to_string();
    let permalink = format!("{nombre_kb}/{}", archivo_rel_str.trim_end_matches(".md"));
    std::fs::create_dir_all(kb_root.join(&dir_archivo))
        .with_context(|| format!("crear {}", dir_archivo.display()))?;
    let documento = construye_archivo(
        plan.preambulo,
        plan.frio,
        &titulo,
        &permalink,
        &archivo_previo,
    );
    escribe_fichero_exclusivo(&kb_root.join(&archivo_rel), &documento)?;
    fsync_directorio(&kb_root.join(&dir_archivo))?;

    let mut salida = Vec::with_capacity(plan.preambulo.len() + aviso.len() + plan.caliente.len());
    salida.extend_from_slice(plan.preambulo);
    salida.extend_from_slice(aviso.as_bytes());
    salida.extend_from_slice(plan.caliente);
    escribe_fichero_atomico(&completa, &salida)?;

    resultado.archivo = Some(archivo_rel_str);
    Ok(resultado)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(entradas: &[&str]) -> Vec<u8> {
        let mut s = String::from("---\ntitle: x-bitacora\ntier: log\n---\n\n# x — bitácora\n\n");
        for e in entradas {
            s.push_str(e);
        }
        s.into_bytes()
    }

    #[test]
    fn mantiene_las_entradas_mas_nuevas_dentro_del_presupuesto() {
        let e1 = format!("## 2026-01-01 — uno\n{}\n\n", "a".repeat(80));
        let e2 = format!("## 2026-02-01 — dos\n{}\n\n", "b".repeat(80));
        let e3 = format!("## 2026-03-01 — tres\n{}\n\n", "c".repeat(80));
        let contenido = doc(&[&e1, &e2, &e3]);

        let p = parte(&contenido, contenido.len() as i64 - 150);

        assert_eq!(p.entradas_calientes, 1);
        assert_eq!(p.entradas_frias, 2);
        assert!(String::from_utf8_lossy(p.caliente).contains("tres"));
        assert!(String::from_utf8_lossy(p.frio).contains("uno"));
        assert!(String::from_utf8_lossy(p.frio).contains("dos"));
        assert!(String::from_utf8_lossy(p.preambulo).starts_with("---\ntitle:"));
        // Invariante duro: nada se pierde.
        let mut reconstruido = Vec::new();
        reconstruido.extend_from_slice(p.preambulo);
        reconstruido.extend_from_slice(p.frio);
        reconstruido.extend_from_slice(p.caliente);
        assert_eq!(reconstruido, contenido);
    }

    #[test]
    fn nada_rota_si_cabe() {
        let contenido = doc(&["## 2026-01-01 — uno\ncorto\n\n"]);
        let p = parte(&contenido, 100_000);
        assert_eq!(p.entradas_frias, 0);
        assert!(p.frio.is_empty());
        assert_eq!(p.entradas_calientes, 1);
    }

    #[test]
    fn siempre_queda_al_menos_una_entrada_caliente_aunque_no_quepa() {
        let vieja = "## 2026-01-01 — vieja\nx\n\n".to_string();
        let gorda = format!("## 2026-03-01 — gorda\n{}\n", "z".repeat(5000));
        let contenido = doc(&[&vieja, &gorda]);
        let p = parte(&contenido, 100);
        assert_eq!(p.entradas_calientes, 1);
        assert!(String::from_utf8_lossy(p.caliente).contains("gorda"));
    }

    #[test]
    fn sin_entradas_todo_queda_en_preambulo() {
        let contenido = b"---\ntier: log\n---\n\n# solo titulo\n".to_vec();
        let p = parte(&contenido, 10);
        assert!(p.frio.is_empty() && p.caliente.is_empty());
        assert_eq!(p.preambulo, &contenido[..]);
    }

    #[test]
    fn un_heading_dentro_de_una_valla_no_es_una_entrada() {
        let e1 = "## 2026-01-01 — uno\n```\n## no soy un heading\n```\n\n".to_string();
        let e2 = "## 2026-02-01 — dos\nx\n\n".to_string();
        let contenido = doc(&[&e1, &e2]);
        let p = parte(&contenido, 100_000);
        assert_eq!(p.entradas_calientes, 2);
    }

    #[test]
    fn nombre_de_archivo_usa_el_rango_de_fechas_iso() {
        let frio = b"## 2026-06-26 -- a\nx\n\n## Actualizacion 2026-07-11 -- b\ny\n\n";
        assert_eq!(
            nombre_de_archivo("agent-develop-bitacora", frio, 0),
            "agent-develop-bitacora-2026-06-26_2026-07-11.md"
        );
    }

    #[test]
    fn nombre_de_archivo_cae_a_numero_de_parte_sin_fechas() {
        let frio = b"## sin fecha\nx\n\n";
        assert_eq!(
            nombre_de_archivo("desarrollo-agentico-bitacora", frio, 2),
            "desarrollo-agentico-bitacora-parte-03.md"
        );
    }

    #[test]
    fn construye_archivo_conserva_tier_log_y_reescribe_title_y_permalink() {
        let fm = b"---\ntitle: agent-develop-bitacora\ntype: note\npermalink: wisdom-paul/log/agent-develop-bitacora\ntags:\n- bitacora\ntier: log\n---\n\n# agent-develop -- bitacora\n\n";
        let frio = b"## 2026-06-26 -- a\nx\n\n";
        let doc = construye_archivo(
            fm,
            frio,
            "agent-develop-bitacora 2026-06-26_2026-07-11",
            "wisdom-paul/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11",
            "",
        );
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.starts_with("---\n"));
        assert!(doc.contains("title: 'agent-develop-bitacora 2026-06-26_2026-07-11'\n"));
        assert!(doc.contains(
            "permalink: 'wisdom-paul/archive/log/agent-develop-bitacora-2026-06-26_2026-07-11'\n"
        ));
        assert!(doc.contains("tier: log\n"));
        assert!(doc.contains("## 2026-06-26"));
    }

    // Pin del finding de la review de kbx: un cierre `---  \n` (espacio
    // final) no debe caer al bloque sintético ni al "title/permalink no se
    // escriben" — la misma clase de bug que motivó el `\r`/espacios en
    // `es_delimitador_frontmatter`.
    #[test]
    fn construye_archivo_tolera_espacio_final_en_el_cierre_del_frontmatter() {
        let fm = b"---\ntype: note\ntags:\n- bitacora\ntier: log\n---  \n\n# x -- bitacora\n\n";
        let frio = b"## 2026-06-26 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "nuevo-titulo", "nuevo-permalink", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.contains("title: 'nuevo-titulo'\n"));
        assert!(doc.contains("permalink: 'nuevo-permalink'\n"));
        assert!(
            doc.contains("type: note\n"),
            "frontmatter original debe conservarse: {doc}"
        );
    }

    #[test]
    fn construye_archivo_cae_a_sintetico_sin_frontmatter_de_origen() {
        let fm = b"# solo un heading\n\ntexto\n";
        let frio = b"## 2026-01-01 -- a\nx\n\n";
        let doc = construye_archivo(fm, frio, "titulo-x", "permalink-x", "");
        let doc = String::from_utf8_lossy(&doc);
        assert!(doc.starts_with("---\ntier: log\n"));
        assert!(doc.contains("title: 'titulo-x'\n"));
    }

    #[test]
    fn construye_archivo_enlaza_al_archivo_previo_cuando_se_pasa() {
        let doc = construye_archivo(
            b"---\ntier: log\n---\n",
            b"## x\n",
            "t",
            "p",
            "bitacora-parte-01",
        );
        assert!(String::from_utf8_lossy(&doc).contains("[[bitacora-parte-01]]"));
    }

    #[test]
    fn frontmatter_sin_terminar_detecta_el_delimitador_sin_cierre() {
        let sin_cerrar = b"---\ntitle: x\nsin cierre aqui\n";
        assert!(frontmatter_sin_terminar(sin_cerrar));
        let normal = b"---\ntier: log\n---\ncuerpo\n";
        assert!(!frontmatter_sin_terminar(normal));
        let sin_frontmatter = b"# solo un heading\n";
        assert!(!frontmatter_sin_terminar(sin_frontmatter));
    }

    fn escribe_nota(root: &std::path::Path, rel: &str, cuerpo: &str) -> std::path::PathBuf {
        let full = root.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, cuerpo).unwrap();
        full
    }

    fn nota_grande() -> String {
        let mut s = String::from(
            "---\ntitle: p-bitacora\ntype: note\npermalink: wisdom-paul/log/p-bitacora\ntier: log\n---\n\n# p -- bitacora\n\n",
        );
        for i in 0..10 {
            s.push_str(&format!("## 2026-0{}-01 -- entrada\n", 1 + i % 9));
            s.push_str(&"x".repeat(3000));
            s.push_str("\n\n");
        }
        s
    }

    #[test]
    fn aplica_en_dry_run_no_toca_disco() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/p-bitacora.md", &nota_grande());
        let antes = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();

        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, false, "wisdom-paul").unwrap();
        assert!(res.rotado);
        let despues = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
        assert_eq!(antes, despues, "dry-run no debe modificar la nota");
        assert!(
            !dir.path().join("archive/log").exists(),
            "dry-run no debe crear archive/"
        );
    }

    #[test]
    fn aplica_escribe_archivo_y_encoge_la_nota() {
        let dir = tempfile::tempdir().unwrap();
        let original = nota_grande();
        escribe_nota(dir.path(), "log/p-bitacora.md", &original);

        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        let caliente = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
        assert!(
            caliente.len() <= 8000 + 200,
            "nota viva demasiado grande: {}",
            caliente.len()
        );
        let frio = std::fs::read(dir.path().join(res.archivo.as_ref().unwrap())).unwrap();
        assert!(String::from_utf8_lossy(&caliente).contains("archivado en"));
        assert!(String::from_utf8_lossy(&frio).contains("tier: log"));

        let total_entradas = String::from_utf8_lossy(&caliente).matches("\n## ").count()
            + String::from_utf8_lossy(&frio).matches("\n## ").count();
        assert_eq!(total_entradas, 10, "nada se borra");
    }

    #[test]
    fn aplica_no_rota_lo_que_ya_cabe() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(
            dir.path(),
            "log/small.md",
            "---\ntier: log\n---\n\n# s\n\n## 2026-01-01 -- u\nx\n",
        );
        let res = aplica(dir.path(), "log/small.md", 100_000, true, "wisdom-paul").unwrap();
        assert!(!res.rotado);
        assert!(!dir.path().join("archive/log").exists());
    }

    #[test]
    fn aplica_rechaza_frontmatter_sin_cerrar() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = String::from("---\ntitle: broken\ntier: log\n\n# broken -- bitacora\n\n");
        for i in 0..10 {
            s.push_str(&format!(
                "## 2026-0{}-01 -- entrada\n{}\n\n",
                1 + i % 9,
                "x".repeat(3000)
            ));
        }
        escribe_nota(dir.path(), "log/broken.md", &s);
        let res = aplica(dir.path(), "log/broken.md", 8000, true, "wisdom-paul");
        assert!(res.is_err());
        assert!(!dir.path().join("archive/log").exists());
        let intacta = std::fs::read_to_string(dir.path().join("log/broken.md")).unwrap();
        assert_eq!(intacta, s, "una nota rechazada no debe tocarse");
    }

    #[test]
    fn aplica_dos_rotaciones_del_mismo_dia_no_se_pisan() {
        let dir = tempfile::tempdir().unwrap();
        let entradas = |n: usize, marca: &str| {
            let mut s = String::new();
            for _ in 0..n {
                s.push_str("## 2026-08-03 -- entrada\n");
                s.push_str(&marca.repeat(300));
                s.push_str("\n\n");
            }
            s
        };
        let original = format!(
            "---\ntitle: x\ntier: log\n---\n\n# x -- bitacora\n\n{}",
            entradas(60, "x")
        );
        escribe_nota(dir.path(), "log/proyecto-x-bitacora.md", &original);

        let r1 = aplica(
            dir.path(),
            "log/proyecto-x-bitacora.md",
            8000,
            true,
            "wisdom-paul",
        )
        .unwrap();
        assert!(r1.rotado);
        let caliente =
            std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        let crecida = caliente + &entradas(40, "y");
        std::fs::write(dir.path().join("log/proyecto-x-bitacora.md"), &crecida).unwrap();

        let r2 = aplica(
            dir.path(),
            "log/proyecto-x-bitacora.md",
            8000,
            true,
            "wisdom-paul",
        )
        .unwrap();
        assert!(r2.rotado);
        assert_ne!(
            r1.archivo, r2.archivo,
            "las dos rotaciones no deben escribir el mismo archivo"
        );

        let frio1 = std::fs::read_to_string(dir.path().join(r1.archivo.unwrap())).unwrap();
        let frio2 = std::fs::read_to_string(dir.path().join(r2.archivo.unwrap())).unwrap();
        let final_caliente =
            std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        let total = final_caliente.matches("\n## ").count()
            + frio1.matches("\n## ").count()
            + frio2.matches("\n## ").count();
        assert_eq!(
            total, 100,
            "nada se pierde ni se pisa entre dos rotaciones del mismo dia"
        );
    }

    #[test]
    fn aplica_reconstruye_byte_a_byte() {
        let dir = tempfile::tempdir().unwrap();
        let original = nota_grande();
        escribe_nota(dir.path(), "log/p-bitacora.md", &original);
        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        assert!(res.rotado);

        let nota_viva = std::fs::read(dir.path().join("log/p-bitacora.md")).unwrap();
        let archivo = std::fs::read(dir.path().join(res.archivo.as_ref().unwrap())).unwrap();
        let preambulo = parte(original.as_bytes(), 8000).preambulo.to_vec();

        assert!(res.bytes_movidos <= archivo.len());
        let frio_del_archivo = &archivo[archivo.len() - res.bytes_movidos..];
        let caliente_esperado = &original.as_bytes()[preambulo.len() + res.bytes_movidos..];
        assert!(caliente_esperado.len() <= nota_viva.len());
        let caliente_de_la_nota = &nota_viva[nota_viva.len() - caliente_esperado.len()..];

        let mut reconstruido = Vec::new();
        reconstruido.extend_from_slice(&preambulo);
        reconstruido.extend_from_slice(frio_del_archivo);
        reconstruido.extend_from_slice(caliente_de_la_nota);
        assert_eq!(reconstruido, original.as_bytes());
    }

    #[test]
    fn aplica_no_deja_temporales_huerfanos() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/p-bitacora.md", &nota_grande());
        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "wisdom-paul").unwrap();
        for entrada in std::fs::read_dir(dir.path().join("log")).unwrap() {
            let nombre = entrada.unwrap().file_name();
            assert!(
                !nombre.to_string_lossy().contains(".tmp"),
                "quedo un temporal: {nombre:?}"
            );
        }
        let dir_archivo = std::path::Path::new(res.archivo.as_ref().unwrap())
            .parent()
            .unwrap();
        for entrada in std::fs::read_dir(dir.path().join(dir_archivo)).unwrap() {
            let nombre = entrada.unwrap().file_name();
            assert!(
                !nombre.to_string_lossy().contains(".tmp"),
                "quedo un temporal: {nombre:?}"
            );
        }
    }

    // Pin de la Decisión D-4: el permalink usa `nombre_kb`, no un literal
    // fijo — `aplica` toma el nombre por parámetro precisamente para que
    // esto sea observable sin montar config.
    #[test]
    fn aplica_usa_el_nombre_de_kb_pasado_como_prefijo_del_permalink() {
        let dir = tempfile::tempdir().unwrap();
        escribe_nota(dir.path(), "log/p-bitacora.md", &nota_grande());
        let res = aplica(dir.path(), "log/p-bitacora.md", 8000, true, "otra-kb").unwrap();
        let archivo = std::fs::read_to_string(dir.path().join(res.archivo.unwrap())).unwrap();
        assert!(
            archivo.contains("permalink: 'otra-kb/archive/log/"),
            "permalink: {archivo}"
        );
    }

    // Pin del Fix 1 de la review: PATRON_AVISO debe casar el aviso real (en
    // UTF-8, no bytes latin1-escapados) o `quita_aviso_previo` es un no-op y
    // los avisos se acumulan en cada rotación.
    #[test]
    fn aplica_dos_veces_seguidas_deja_un_solo_aviso_y_encadena_los_archivos() {
        let dir = tempfile::tempdir().unwrap();
        let entradas = |n: usize, marca: &str| {
            let mut s = String::new();
            for _ in 0..n {
                s.push_str("## 2026-08-03 -- entrada\n");
                s.push_str(&marca.repeat(300));
                s.push_str("\n\n");
            }
            s
        };
        let original = format!(
            "---\ntitle: x\ntier: log\n---\n\n# x -- bitacora\n\n{}",
            entradas(60, "x")
        );
        escribe_nota(dir.path(), "log/proyecto-x-bitacora.md", &original);

        let r1 = aplica(
            dir.path(),
            "log/proyecto-x-bitacora.md",
            8000,
            true,
            "wisdom-paul",
        )
        .unwrap();
        assert!(r1.rotado);
        let caliente =
            std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        let crecida = caliente + &entradas(40, "y");
        std::fs::write(dir.path().join("log/proyecto-x-bitacora.md"), &crecida).unwrap();

        let r2 = aplica(
            dir.path(),
            "log/proyecto-x-bitacora.md",
            8000,
            true,
            "wisdom-paul",
        )
        .unwrap();
        assert!(r2.rotado);

        let nota_viva =
            std::fs::read_to_string(dir.path().join("log/proyecto-x-bitacora.md")).unwrap();
        assert_eq!(
            nota_viva.matches("archivado en").count(),
            1,
            "la nota viva debe llevar exactamente un aviso tras dos rotaciones: {nota_viva}"
        );

        let archivo1 = r1.archivo.unwrap();
        let archivo2 = r2.archivo.unwrap();
        let nombre1_sin_md = std::path::Path::new(&archivo1)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let contenido_archivo2 = std::fs::read_to_string(dir.path().join(&archivo2)).unwrap();
        assert!(
            contenido_archivo2.contains(&format!("[[{nombre1_sin_md}]]")),
            "archivo2 debe enlazar hacia atrás a archivo1 ({nombre1_sin_md}): {contenido_archivo2}"
        );
    }

    // Pin de I1 (segunda vuelta de review): `escribe_temporal` deja el
    // temporal ya escrito y fsyncado; publicarlo es responsabilidad de
    // cada llamador (`rename` aquí, `hard_link` en el exclusivo de abajo).
    // Este test cubre el camino de error que NO pasa por `escribe_temporal`
    // — `rename` falla porque `ruta` ya es un directorio (EISDIR/ENOTDIR
    // según plataforma) — y confirma que la limpieza de
    // `escribe_fichero_atomico` (`if resultado.is_err() { remove_file }`)
    // no deja el `.tmp` huérfano.
    #[test]
    fn escribe_fichero_atomico_borra_el_tmp_si_falla_el_rename() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("ya-es-un-directorio.md");
        std::fs::create_dir_all(&ruta).unwrap();

        let resultado = escribe_fichero_atomico(&ruta, b"contenido nuevo");

        assert!(
            resultado.is_err(),
            "rename sobre un directorio existente debe fallar"
        );
        for entrada in std::fs::read_dir(dir.path()).unwrap() {
            let nombre = entrada.unwrap().file_name();
            assert!(
                !nombre.to_string_lossy().contains(".tmp"),
                "quedo un temporal huerfano tras el rename fallido: {nombre:?}"
            );
        }
    }

    // Pin de I1 (segunda vuelta de review): mismo escenario que el test de
    // arriba pero para `escribe_fichero_exclusivo` — `hard_link` falla
    // porque `ruta` ya existe (EEXIST, el caso que este backstop existe
    // para detectar), y la limpieza incondicional
    // (`let _ = remove_file(&tmp); resultado`) no debe dejar el `.tmp`
    // huérfano ni tocar el fichero ya existente.
    #[test]
    fn escribe_fichero_exclusivo_borra_el_tmp_si_falla_el_hard_link() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("ya-existe.md");
        std::fs::write(&ruta, b"contenido original").unwrap();

        let resultado = escribe_fichero_exclusivo(&ruta, b"contenido nuevo");

        assert!(
            resultado.is_err(),
            "hard_link no debe pisar un fichero ya existente"
        );
        for entrada in std::fs::read_dir(dir.path()).unwrap() {
            let nombre = entrada.unwrap().file_name();
            assert!(
                !nombre.to_string_lossy().contains(".tmp"),
                "quedo un temporal huerfano tras el hard_link fallido: {nombre:?}"
            );
        }
        let intacto = std::fs::read(&ruta).unwrap();
        assert_eq!(
            intacto, b"contenido original",
            "el fichero existente no debe tocarse cuando hard_link falla"
        );
    }
}
