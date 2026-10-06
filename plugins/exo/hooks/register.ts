const FRAMING =
  "## Reglas duras del proyecto\nEstas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.\n\n";

type Latido = { status: "ok" | "skip" | "error"; reason?: string; n: number; error?: string };

// Cada intento se rinde a error, nunca lanza: compose no puede romper el arranque de la sesión.
async function consultar($: any, home: string, cwd: string): Promise<{ hb: Latido; reglas: string[] }> {
  const vacio: string[] = [];
  try {
    const run = (bin: string) => $.process.run([bin, "rules", "--cwd", cwd, "--json"], { timeoutMs: 3000 });
    let res: any;
    try {
      res = await run("exo");
    } catch {
      // process.run rechaza cuando el binario no arranca (PATH sin exo)
      res = await run(`${home}/.local/bin/exo`);
    }
    if (res.exitCode !== 0) {
      const detalle = String(res.stderr || res.stdout || "").trim().slice(0, 200);
      return { hb: { status: "error", n: 0, error: `exit ${res.exitCode}${detalle ? `: ${detalle}` : ""}` }, reglas: vacio };
    }
    const data = JSON.parse(res.stdout).data;
    if (data?.status === "ok" && Array.isArray(data.rules)) {
      return { hb: { status: "ok", n: data.rules.length }, reglas: data.rules.map(String) };
    }
    if (data?.status === "skip") {
      return { hb: { status: "skip", reason: String(data.reason), n: 0 }, reglas: vacio };
    }
    return { hb: { status: "error", n: 0, error: "respuesta sin data.status ok|skip" }, reglas: vacio };
  } catch (err: any) {
    return { hb: { status: "error", n: 0, error: String(err?.message || err) || "error" }, reglas: vacio };
  }
}

export function register(on: any) {
  on("prompt.compose", async ($: any, e: any, next: any) => {
    const r = await next(e);
    let hb: Latido = { status: "error", n: 0, error: "mod" };
    let reglas: string[] = [];
    let sid = "";
    let home = "";
    try {
      sid = await $.session.id();
      home = (await $.env.get("HOME")) || (await $.env.get("USERPROFILE")) || "";
      ({ hb, reglas } = await consultar($, home, await $.session.cwd()));
    } catch (err: any) {
      hb = { status: "error", n: 0, error: String(err?.message || err) || "error" };
    }
    // Sin sid o sin HOME no hay nombre de latido válido: no se escribe en una ruta inventada.
    if (!sid || !home) return r;
    try {
      await $.fs.write(`${home}/.claude/exo-rules/hb-${sid}`, JSON.stringify(hb));
    } catch {
      return r;
    }
    if (hb.status !== "ok" || reglas.length === 0) return r;
    const text = FRAMING + reglas.map((x) => `- ${x}`).join("\n");
    return { sections: [...r.sections, { id: "exo:reglas-proyecto", text, scope: "session" }] };
  });
}
