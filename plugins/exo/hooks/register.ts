const FRAMING =
  "## Reglas duras del proyecto\nEstas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.\n\n";

type Latido = {
  status: "ok" | "skip" | "error";
  reason?: string;
  n: number;
  error?: string;
  via: "compose" | "submit" | "none";
  org?: string;
};

const SIN_ENTREGA: Latido = { status: "error", n: 0, via: "none", error: "sin entrega" };

// Estado por sid (no global): /clear cambia el sid sin recargar el mod.
const composeVivo = new Set<string>();
const turnos = new Map<string, number>();
const composeMarcado = new Set<string>();
const submitMarcado = new Set<string>();

const hbPath = (home: string, sid: string) => `${home}/.claude/exo-rules/hb-${sid}`;
const bloque = (reglas: string[]) => FRAMING + reglas.map((x) => `- ${x}`).join("\n");
const msg = (err: any) => String(err?.message || err) || "error";

// Pista no documentada y puede rechazar (>4 MiB): nunca gate, nunca lanza.
// Se cachea por proceso solo el valor leído; un fallo se reintenta en el siguiente turno.
let orgCache: string | undefined;
async function leerOrg($: any, home: string): Promise<string | undefined> {
  if (orgCache) return orgCache;
  try {
    const t = JSON.parse(await $.fs.read(`${home}/.claude.json`));
    const org = t?.oauthAccount?.organizationType;
    if (typeof org === "string" && org) orgCache = org;
    return orgCache;
  } catch {
    return undefined;
  }
}

// Cada intento se rinde a error, nunca lanza: el mod no puede romper el arranque de la sesión ni el prompt.
async function consultar($: any, home: string, cwd: string): Promise<{ hb: Omit<Latido, "via">; reglas: string[] }> {
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
    return { hb: { status: "error", n: 0, error: msg(err) }, reglas: vacio };
  }
}

async function escribirHb($: any, home: string, sid: string, hb: Latido): Promise<void> {
  try {
    await $.fs.write(hbPath(home, sid), JSON.stringify(hb));
  } catch {
    // un latido no escrito no debe impedir la entrega
  }
}

// Devuelve el bloque a adjuntar o null; las escrituras de latido/store van dentro y nunca lanzan hacia fuera.
async function fallbackSubmit($: any): Promise<string | null> {
  const sid = await $.session.id();
  const home = (await $.env.get("HOME")) || (await $.env.get("USERPROFILE")) || "";
  if (!sid || !home) return null;
  const forzar = (await $.env.get("EXO_RULES_FORZAR_SUBMIT")) === "1";
  const turno = (turnos.get(sid) ?? 0) + 1;
  turnos.set(sid, turno);
  const vivo = composeVivo.has(sid);
  const storeCompose = forzar ? undefined : await $.store.get("compose").catch(() => undefined);
  const entregar = forzar || (!vivo && (turno >= 2 || storeCompose !== true));

  if (!entregar) {
    if (!vivo && !(await $.fs.exists(hbPath(home, sid)).catch(() => false))) {
      const org = await leerOrg($, home);
      await escribirHb($, home, sid, { ...SIN_ENTREGA, ...(org ? { org } : {}) });
    }
    return null;
  }

  const { hb, reglas } = await consultar($, home, await $.session.cwd());
  const org = await leerOrg($, home);
  await escribirHb($, home, sid, { ...hb, via: "submit", ...(org ? { org } : {}) });
  if (!forzar && turno >= 2 && storeCompose !== false && !submitMarcado.has(sid)) {
    submitMarcado.add(sid);
    await $.store.set("compose", false).catch(() => {});
  }
  return hb.status === "ok" && reglas.length > 0 ? bloque(reglas) : null;
}

export function register(on: any) {
  on("session.start", async ($: any, e: any, next: any) => {
    try {
      const sid = await $.session.id();
      const home = (await $.env.get("HOME")) || (await $.env.get("USERPROFILE")) || "";
      if (sid && home) {
        const org = await leerOrg($, home);
        if (org) await $.store.set("org", org).catch(() => {});
        if (!(await $.fs.exists(hbPath(home, sid)))) await escribirHb($, home, sid, SIN_ENTREGA);
      }
    } catch {
      // siembra best-effort: sin ella el testigo ve "no cargó" en vez de "sin canal"
    }
    return next(e);
  });

  on("prompt.submit", async ($: any, e: any, next: any) => {
    let extra: string | null = null;
    try {
      extra = await fallbackSubmit($);
    } catch {
      extra = null;
    }
    return next(extra ? { ...e, context: [...(e.context ?? []), extra] } : e);
  }).catch(($: any, e: any, next: any) => next(e));

  // Sin compose vivo en este sid, la marca del store es de otra cuenta/sesión: se baja para que
  // las sesiones de un solo prompt (-p) no queden sin reglas tras pasar de cuenta personal a Team.
  on("turn.complete", async ($: any, e: any, next: any) => {
    try {
      const sid = await $.session.id();
      if (sid && !composeVivo.has(sid) && (await $.env.get("EXO_RULES_FORZAR_SUBMIT")) !== "1") {
        if ((await $.store.get("compose")) === true) await $.store.set("compose", false);
      }
    } catch {
      // best-effort: sin bajar la marca solo se pierde la autocuración
    }
    return next(e);
  });

  on("prompt.compose", async ($: any, e: any, next: any) => {
    // Seam de test: con FORZAR compose queda inerte para que el eval/e2e midan solo el canal submit.
    try {
      if ((await $.env.get("EXO_RULES_FORZAR_SUBMIT")) === "1") return await next(e);
    } catch {
      // sin env legible se sigue por el camino normal
    }
    const r = await next(e);
    let hb: Latido = { status: "error", n: 0, via: "compose", error: "mod" };
    let reglas: string[] = [];
    let sid = "";
    let home = "";
    try {
      sid = await $.session.id();
      if (sid) composeVivo.add(sid);
      home = (await $.env.get("HOME")) || (await $.env.get("USERPROFILE")) || "";
      const c = await consultar($, home, await $.session.cwd());
      hb = { ...c.hb, via: "compose" };
      reglas = c.reglas;
    } catch (err: any) {
      hb = { status: "error", n: 0, via: "compose", error: msg(err) };
    }
    // Sin sid o sin HOME no hay nombre de latido válido: no se escribe en una ruta inventada.
    if (!sid || !home) return r;
    try {
      await $.fs.write(hbPath(home, sid), JSON.stringify(hb));
    } catch {
      return r;
    }
    if (!composeMarcado.has(sid)) {
      composeMarcado.add(sid);
      await $.store.set("compose", true).catch(() => {});
    }
    if (hb.status !== "ok" || reglas.length === 0) return r;
    return { sections: [...r.sections, { id: "exo:reglas-proyecto", text: bloque(reglas), scope: "session" }] };
  });
}
