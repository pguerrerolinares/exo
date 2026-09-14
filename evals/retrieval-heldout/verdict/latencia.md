# Latencia — campaña C (pre-registro §7, protocolo R3)

- Binario: `$PRIV/exo-medicion` (commit en `condiciones.md`); query fija `memoria persistente de sesiones`; `hyperfine --warmup 3 --runs 30`, índices corridos consecutivamente.

| índice | p95 (s) | chunks_embedded (rebuild) | rebuild (s) | RSS pico rebuild |
|---|---|---|---|---|
| base | 1.0240 | 3307 | 5335 | 6172660 kB |
| solape | 1.0258 | 3448 | 2752 | 5761740 kB |
| late | no medido (D3 = no) | — | — | — |

- Guard R3: p95 solape ≤ 1,25 × p95 base → sí (1.0258 ≤ 1.2800)
- Guard R4: rebuild late ≤ 3 × rebuild base → no medido
- Intento 1 descartado antes de correr el informe: la serie de `solape` salió bimodal (10 runs a ~8,9 s y 20 a ~0,95 s; 62 min de reloj para 30 runs), y la de `base` estable a ~1,94 s. Se incumplía la condición del plan "sin otra carga pesada en la máquina" (cambio de estado del equipo a mitad de la medición). El intento 2, con el mismo protocolo, sale estacionario en ambos índices (máx 1.041 y 1.032 s). Ficheros de los dos intentos en el directorio privado.
- Los tiempos de rebuild no son comparables entre índices: dependen de la carga de la máquina en cada momento (`base`, con menos trozos, tardó casi el doble). Ninguna regla de C los usa (R4 no se mide).
