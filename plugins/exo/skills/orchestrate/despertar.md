# Despertar de orquestación

Te ha despertado un cron de respaldo, no un evento. Haz, en este orden y sin narrar:

1. Para cada tarea de fondo que esperabas: mira su output (ruta en el brief o en el ledger; `mtime`, contenido final).
2. Output terminado y su dueño es un teammate idle → `SendMessage` al teammate con la ruta del output y «tu tarea terminó; sigue y entrega el report».
3. Output terminado y el dueño eres tú → sigue con el siguiente paso del ledger.
4. Tarea aún viva → arma UN cron one-shot más a +N (N ≤ 20) y cierra turno.
5. Lo único pendiente es un gate humano → no hagas nada; la pregunta ya está en el ledger.
6. Una línea en el ledger: hora, qué encontraste, qué hiciste.
