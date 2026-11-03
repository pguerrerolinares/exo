use exo::state::{Migracion, migra};
use std::fs;

// Binario propio: el contador de reclamos es global al proceso y arranca en 0 solo si ningún
// otro test migra antes. Con un único test, el primer nombre candidato es `-0`.
#[test]
fn reclamo_sobrante_con_el_mismo_nombre_no_se_pisa() {
    let d = tempfile::tempdir().unwrap();
    let (v, n) = (d.path().join("v"), d.path().join("n"));
    let sobrante = |i: usize| {
        d.path()
            .join(format!("v.migrando-{}-{i}", std::process::id()))
    };
    for i in 0..4 {
        fs::write(sobrante(i), "X\n").unwrap();
    }
    fs::write(&v, "Y\n").unwrap();
    assert_eq!(migra(&v, &n, "20261103T100000"), Migracion::Renombrado);
    assert_eq!(fs::read_to_string(&n).unwrap(), "Y\n");
    for i in 0..4 {
        assert_eq!(
            fs::read_to_string(sobrante(i)).unwrap(),
            "X\n",
            "sobrante {i}"
        );
    }
}
