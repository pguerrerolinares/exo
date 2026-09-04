//! El gate de dominio: "la KB está mal" no es "el binario falló".
//!
//! `escritor::Rechazo` no vale aquí: lleva un `data()` que `main` emite como
//! envelope en la rama de error, porque en `write` el detalle del rechazo ES la
//! respuesta. `budget` y `lint` ya han emitido su informe completo antes de
//! gatear, así que solo necesitan el exit code y una línea a stderr.
#[derive(Debug)]
pub struct GateFallido {
    pub comando: &'static str,
    pub detalle: String,
}

impl std::fmt::Display for GateFallido {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.comando, self.detalle)
    }
}

impl std::error::Error for GateFallido {}
