use anyhow::{Context, Result};
use rusqlite::Connection;

/// DDL del índice propio de exo (spec `2026-07-17-indexer-design.md` §2, VERBATIM).
/// Nombres en castellano, deliberadamente distintos de las tablas de basic-memory
/// (veto AGPL: ni una línea de su código, solo forma de schema — de dominio público).
/// Idempotente: `CREATE TABLE IF NOT EXISTS` / `CREATE VIRTUAL TABLE IF NOT EXISTS`.
/// Aristas/trozos/vectores se crean aquí pero se pueblan en M2-04/M2-06.
pub fn crea_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS notas (
          permalink  TEXT PRIMARY KEY,
          ruta       TEXT NOT NULL UNIQUE,
          titulo     TEXT NOT NULL,
          tipo       TEXT,
          mtime      REAL NOT NULL,
          git_epoch  INTEGER
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS notas_fts USING fts5(
          titulo, cuerpo,
          permalink UNINDEXED,
          tokenize='unicode61 tokenchars 0x2F'
        );

        CREATE TABLE IF NOT EXISTS aristas (
          origen            TEXT NOT NULL REFERENCES notas(permalink),
          destino_texto     TEXT NOT NULL,
          destino_permalink TEXT,
          UNIQUE (origen, destino_texto)
        );

        CREATE TABLE IF NOT EXISTS trozos (
          id        INTEGER PRIMARY KEY,
          permalink TEXT NOT NULL REFERENCES notas(permalink),
          orden     INTEGER NOT NULL,
          texto     TEXT NOT NULL,
          UNIQUE (permalink, orden)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS vectores USING vec0(embedding float[768]);
        ",
    )
    .context("crear schema del índice")
}
