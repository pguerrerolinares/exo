# Lanzar Ap (preregistro.md + erratas.md E1)

Gasta API (~25 USD, 280 corridas). Solo con el visto bueno del dueño.

```bash
export K_ROOT=~/.cache/exo-campana-m
# 1. Las 40 sondas de M se reconstruyen con su setup.sh determinista; M no usó este fichero.
cut -f1 evals/ap/tareas.tsv | sed 's/$/\tsi/' > "$K_ROOT/reconstruccion.tsv"
# 2. Bloques desde la KB fijada (552f06a); tienen que casar con bloques.sha256 y dar N=16.
git -C ~/Documentos/proyectos/wisdom-paul rev-parse --short HEAD   # = 552f06a, o usa un worktree en ese commit
python3 evals/ap/generar-bloques.py --kb ~/Documentos/proyectos/wisdom-paul
(cd "$K_ROOT" && sha256sum -c --quiet "$OLDPWD/evals/ap/bloques.sha256")
git diff --exit-code evals/ap/cobertura.tsv evals/ap/tareas.tsv     # nada ha cambiado
# 3. Ensayo sin claude: 280 lanzadas, 0 sin cmdline.txt.
K_ENSAYO=1 TECHO_EXP=evals/ap bash evals/techo-reglas/correr-techo.sh 4
# 4. Tanda real: versión de claude, cero re-intentos y breakers de correr-techo (pins.sha256 verifica el gold).
claude --version > evals/ap/claude-version.txt   # commitear ANTES de lanzar
K_TOPE_USD=30 TECHO_EXP=evals/ap bash evals/techo-reglas/correr-techo.sh 4
# 5. Adjudicar.
python3 evals/ap/evaluar-ap.py "$K_ROOT" evals/ap
```
