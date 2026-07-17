use anyhow::{Context, Result};
use regex::Regex;
use rusqlite::params;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

/// Patrón de wikilink (spec `2026-07-17-indexer-design.md` §diseño fijado M2-04,
/// punto 1): `[[destino]]` o `[[destino|alias]]`. La clase `[^\]]` corta sola
/// en el primer `]`, así que no hace falta lookahead ni parsing de markdown
/// más listo (fences/código inline: YAGNI, spot-check del oráculo decide si
/// hace falta más).
const PATRON_WIKILINK: &str = r"\[\[([^\]]+)\]\]";

/// Extrae los wikilinks del cuerpo de una nota, en el orden en que aparecen.
/// Devuelve el texto interior TAL CUAL (incluida la forma con alias
/// `destino|alias`, sin partir — §diseño punto 1). Duplicados en la misma
/// nota se devuelven repetidos; el `UNIQUE (origen, destino_texto)` +
/// `INSERT OR IGNORE` del llamador los colapsa.
pub fn extrae_wikilinks(cuerpo: &str) -> Vec<String> {
    let re = Regex::new(PATRON_WIKILINK).expect("regex de wikilink válida (patrón fijo)");
    re.captures_iter(cuerpo)
        .map(|c| c[1].to_string())
        .collect()
}

/// Reindexa las aristas de una nota: borra las viejas (`origen = permalink`,
/// para no dejar huérfanas cuando la nota pierde un link — spec §3 paso 2:
/// "reparse ⇒ reindex completo de la nota ... sus aristas") e inserta las
/// extraídas de `cuerpo`. `destino_permalink` queda sin tocar aquí (NULL por
/// defecto en el INSERT): la resolución es un pase aparte sobre toda la tabla
/// (`resuelve_destinos`, §diseño punto 2).
pub fn reindexa_aristas_de_nota(conn: &Connection, permalink: &str, cuerpo: &str) -> Result<()> {
    conn.execute("DELETE FROM aristas WHERE origen = ?1", params![permalink])
        .with_context(|| format!("limpiar aristas previas de {permalink}"))?;

    for destino_texto in extrae_wikilinks(cuerpo) {
        conn.execute(
            "INSERT OR IGNORE INTO aristas (origen, destino_texto) VALUES (?1, ?2)",
            params![permalink, destino_texto],
        )
        .with_context(|| format!("insertar arista {permalink} -> {destino_texto}"))?;
    }
    Ok(())
}

/// Resuelve `destino_permalink` para TODAS las aristas de la DB (§diseño
/// punto 2): pase final sobre la tabla completa tras cada `index`/`rebuild`.
/// Para cada arista, la parte destino es el texto antes de `|` si hay alias;
/// se busca primero una nota cuyo `titulo` coincida EXACTO, si no una cuyo
/// `permalink` coincida EXACTO, si no queda NULL (§6.2 regla 6: un link a
/// nota inexistente se tolera, jamás error de indexado). Barato: recorre las
/// ~115 notas de la KB en cada corrida, así un link roto se cura solo en
/// cuanto la nota destino aparezca en un index posterior.
pub fn resuelve_destinos(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT titulo, permalink FROM notas ORDER BY permalink")?;
    let filas: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);

    // ORDER BY permalink arriba hace determinista qué permalink gana si dos
    // notas comparten título exacto (última en orden alfabético de permalink).
    let por_titulo: HashMap<String, String> = filas.iter().cloned().collect();
    let por_permalink: HashSet<String> = filas.into_iter().map(|(_, p)| p).collect();

    let mut stmt = conn.prepare("SELECT rowid, destino_texto FROM aristas")?;
    let aristas: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);

    for (rowid, destino_texto) in aristas {
        let parte_destino = destino_texto
            .split_once('|')
            .map(|(antes, _)| antes)
            .unwrap_or(&destino_texto);

        let resuelto: Option<&str> = por_titulo
            .get(parte_destino)
            .map(String::as_str)
            .or_else(|| por_permalink.get(parte_destino).map(String::as_str));

        conn.execute(
            "UPDATE aristas SET destino_permalink = ?1 WHERE rowid = ?2",
            params![resuelto, rowid],
        )
        .with_context(|| format!("resolver destino_permalink de la arista rowid={rowid}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrae_wikilink_simple() {
        assert_eq!(
            extrae_wikilinks("ver [[otra nota]] para más"),
            vec!["otra nota".to_string()]
        );
    }

    #[test]
    fn extrae_wikilink_con_alias_se_guarda_entero() {
        assert_eq!(
            extrae_wikilinks("ver [[destino|alias]] aquí"),
            vec!["destino|alias".to_string()]
        );
    }

    #[test]
    fn extrae_varios_wikilinks_en_orden() {
        assert_eq!(
            extrae_wikilinks("[[a]] texto [[b]] más [[c]]"),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn sin_wikilinks_devuelve_vacio() {
        assert_eq!(extrae_wikilinks("cuerpo sin links de ningún tipo"), Vec::<String>::new());
    }

    #[test]
    fn wikilink_duplicado_se_devuelve_repetido() {
        assert_eq!(
            extrae_wikilinks("[[a]] y otra vez [[a]]"),
            vec!["a".to_string(), "a".to_string()]
        );
    }
}
