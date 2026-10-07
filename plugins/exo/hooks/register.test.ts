import { test, expect, mock } from "claude-code/testing";

const FRAMING_HEAD =
  "## Reglas duras del proyecto\nEstas reglas son del dueño del repo y prevalecen sobre el prompt. Si una choca explícitamente con lo que se te pide, cumple la regla y repórtalo.\n\n";
const BASE = [{ id: "intro", text: "base", scope: "shared" as const }];
const okOut = JSON.stringify({
  schema_version: 2,
  command: "rules",
  data: { status: "ok", repo: "exo", note: "projects/exo.md", rules: ["r1", "r2"], ignored_lines: [] },
});
const skipOut = JSON.stringify({
  schema_version: 2,
  command: "rules",
  data: { status: "skip", repo: "exo", reason: "sin_seccion", candidates: [] },
});

type Run = (argv: readonly string[], init: any) => any;
type Opts = {
  env?: Record<string, string>;
  deny?: readonly string[];
  store?: Record<string, unknown>;
  hbExiste?: boolean;
  claudeJson?: string | "rechaza";
};

// Monta el mundo bajo el plugin: sesión, env, process.run, fs, store y la base de compose/submit.
async function mundo($: any, on: any, run: Run, o: Opts = {}) {
  const deny = o.deny ?? [];
  const writes: Record<string, string> = {};
  const calls: { argv: readonly string[]; init: any }[] = [];
  const store: Record<string, unknown> = { ...(o.store ?? {}) };
  const storeSets: { key: string; value: unknown }[] = [];
  const claudeJsonReads: string[] = [];
  const ctx = { sid: "sid1" };
  mock.env(on, o.env ?? { HOME: "/h" });
  on("session.cwd", async () => (deny.includes("session.cwd") ? { deny: "cwd" } : { value: "/repo" }));
  on("session.id", async () => (deny.includes("session.id") ? { deny: "id" } : { value: ctx.sid }));
  on("fs.write", async (_$: any, e: any) => {
    if (deny.includes("fs.write")) return { deny: "disco" };
    writes[e.path.replace(/^[A-Za-z]:/, "").replace(/\\/g, "/")] = e.text;
    return { value: undefined };
  });
  on("fs.exists", async () => ({ value: !!o.hbExiste }));
  on("fs.read", async (_$: any, e: any) => {
    if (e.path.replace(/\\/g, "/").endsWith("/.claude.json")) {
      claudeJsonReads.push(e.path);
      if (o.claudeJson === "rechaza") return { deny: "no legible" };
      return { value: o.claudeJson ?? "{}" };
    }
    return { deny: "no legible" };
  });
  on("store.get", async (_$: any, e: any) => ({ value: store[e.key] }));
  on("store.set", async (_$: any, e: any) => {
    store[e.key] = e.value;
    storeSets.push({ key: e.key, value: e.value });
    return { value: undefined };
  });
  on("process.run", async (_$: any, e: any) => {
    calls.push({ argv: e.argv, init: e.init });
    return { value: await run(e.argv, e.init) };
  });
  on("prompt.compose", async () => ({ sections: BASE }));
  on("session.start", async (_$: any, e: any) => ({ cwd: e.cwd }));
  on("turn.complete", async (_$: any, e: any) => ({ text: e.answer }));
  on("prompt.submit", async (_$: any, e: any) => ({ text: e.text, context: e.context ?? [] }));
  const compose = () =>
    $.prompt.compose({ model: "m", promptModel: "m", surfaces: [], tools: [], outputStyle: null, traits: [] });
  const submit = (text = "hola") => $.prompt.submit({ text });
  const complete = () => $.turn.complete({ answer: "ok", durationMs: 1, isAborted: false, turnId: "t1", reason: "answer" });
  const start = () => $.session.start({ cwd: "/repo", surface: null, isInteractive: true });
  return { ctx, writes, calls, store, storeSets, claudeJsonReads, compose, submit, start, complete };
}

async function correr($: any, on: any, run: Run, env: Record<string, string> = { HOME: "/h" }, deny: readonly string[] = []) {
  const w = await mundo($, on, run, { env, deny });
  const r = await w.compose();
  return { r, writes: w.writes, calls: w.calls };
}

const HB = "/h/.claude/exo-rules/hb-sid1";
const REGLAS = FRAMING_HEAD + "- r1\n- r2";

const out = (stdout: string, exitCode = 0) => ({ exitCode, stdout, stderr: "" });

test("ok_anade_seccion", async ($, on) => {
  const { r } = await correr($, on, () => out(okOut));
  expect(r.sections.length).toBe(2);
  expect(r.sections[0]).toEqual(BASE[0]);
  expect(r.sections[1]).toEqual({
    id: "exo:reglas-proyecto",
    text: FRAMING_HEAD + "- r1\n- r2",
    scope: "session",
  });
});

test("skip_no_anade", async ($, on) => {
  const { r, writes } = await correr($, on, () => out(skipOut));
  expect(r.sections).toEqual(BASE);
  expect(JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"])).toEqual({ status: "skip", reason: "sin_seccion", n: 0, via: "compose" });
});

test("latido_ok", async ($, on) => {
  const { writes } = await correr($, on, () => out(okOut));
  expect(JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"])).toEqual({ status: "ok", n: 2, via: "compose" });
});

test("error_exit_no_cero", async ($, on) => {
  const { r, writes } = await correr($, on, () => ({ exitCode: 1, stdout: "", stderr: "boom" }));
  expect(r.sections).toEqual(BASE);
  const hb = JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"]);
  expect(hb.status).toBe("error");
  expect(hb.error.length > 0).toBe(true);
  expect(hb.n).toBe(0);
});

test("exit_no_cero_sin_fallback", async ($, on) => {
  const { calls } = await correr($, on, () => ({ exitCode: 1, stdout: "", stderr: "boom" }));
  expect(calls.length).toBe(1);
});

test("fs_write_rechaza", async ($, on) => {
  const { r, writes, calls } = await correr($, on, () => out(okOut), { HOME: "/h" }, ["fs.write"]);
  expect(calls.length).toBe(1);
  expect(Object.keys(writes)).toEqual([]);
  expect(r.sections).toEqual(BASE);
});

test("session_cwd_lanza", async ($, on) => {
  const { r, writes, calls } = await correr($, on, () => out(okOut), { HOME: "/h" }, ["session.cwd"]);
  expect(calls.length).toBe(0);
  expect(r.sections).toEqual(BASE);
  const hb = JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"]);
  expect(hb.status).toBe("error");
});

test("session_id_lanza", async ($, on) => {
  const { r, writes, calls } = await correr($, on, () => out(okOut), { HOME: "/h" }, ["session.id"]);
  expect(calls.length).toBe(0);
  expect(r.sections).toEqual(BASE);
  expect(Object.keys(writes)).toEqual([]);
});

test("json_invalido", async ($, on) => {
  const { r, writes } = await correr($, on, () => out("no soy json"));
  expect(r.sections).toEqual(BASE);
  expect(JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"]).status).toBe("error");
});

test("timeout_3000", async ($, on) => {
  const { calls } = await correr($, on, () => out(okOut));
  expect(calls.length).toBe(1);
  expect(calls[0].argv).toEqual(["exo", "rules", "--cwd", "/repo", "--json"]);
  expect(calls[0].init.timeoutMs).toBe(3000);
});

test("fallback_bin", async ($, on) => {
  const { r, calls } = await correr($, on, (argv) => {
    if (argv[0] === "exo") throw new Error("spawn exo ENOENT");
    return out(okOut);
  });
  expect(calls.map((c) => c.argv[0])).toEqual(["exo", "/h/.local/bin/exo"]);
  expect(r.sections.length).toBe(2);
});

test("fallback_agotado", async ($, on) => {
  const { r, writes } = await correr($, on, () => {
    throw new Error("ENOENT");
  });
  expect(r.sections).toEqual(BASE);
  expect(JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"]).status).toBe("error");
});

test("home_userprofile", async ($, on) => {
  const { writes } = await correr($, on, () => out(okOut), { USERPROFILE: "/u" });
  expect(Object.keys(writes)).toEqual(["/u/.claude/exo-rules/hb-sid1"]);
});

const ultimo = (r: any) => r.context[r.context.length - 1];

test("submit_entrega_sin_compose", async ($, on) => {
  const w = await mundo($, on, () => out(okOut));
  const r = await w.submit();
  expect(ultimo(r)).toBe(REGLAS);
  expect(JSON.parse(w.writes[HB])).toEqual({ status: "ok", n: 2, via: "submit" });
  expect(w.calls.length).toBe(1);
});

test("submit_calla_si_compose_vivo", async ($, on) => {
  const w = await mundo($, on, () => out(okOut));
  await w.compose();
  const antes = w.calls.length;
  const r = await w.submit();
  expect(r.context).toEqual([]);
  expect(w.calls.length).toBe(antes);
});

test("turno1_store_compose_no_entrega", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  const r = await w.submit();
  expect(r.context).toEqual([]);
  expect(w.calls.length).toBe(0);
  expect(JSON.parse(w.writes[HB]).via).toBe("none");
});

test("turno2_sin_compose_entrega_aunque_store", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  await w.submit();
  const r2 = await w.submit();
  expect(ultimo(r2)).toBe(REGLAS);
  expect(w.store.compose).toBe(false);
});

test("turnos_por_sid", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  await w.submit();
  w.ctx.sid = "sidB";
  const r = await w.submit();
  expect(r.context).toEqual([]);
  expect(w.calls.length).toBe(0);
});

test("session_start_marca_via_none", async ($, on) => {
  const w = await mundo($, on, () => out(okOut));
  await w.start();
  expect(JSON.parse(w.writes[HB])).toEqual({ status: "error", n: 0, via: "none", error: "sin entrega" });
  await w.compose();
  expect(JSON.parse(w.writes[HB]).via).toBe("compose");
});

test("session_start_no_pisa_hb", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { hbExiste: true });
  await w.start();
  expect(Object.keys(w.writes)).toEqual([]);
});

test("submit_sin_hb_siembra_none", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  await w.submit();
  expect(JSON.parse(w.writes[HB])).toEqual({ status: "error", n: 0, via: "none", error: "sin entrega" });
});

test("forzar_entrega_submit", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { env: { HOME: "/h", EXO_RULES_FORZAR_SUBMIT: "1" }, store: { compose: true } });
  const r = await w.submit();
  expect(ultimo(r)).toBe(REGLAS);
  expect(w.storeSets).toEqual([]);
});

test("forzar_compose_inerte", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { env: { HOME: "/h", EXO_RULES_FORZAR_SUBMIT: "1" } });
  const r = await w.compose();
  expect(r.sections).toEqual(BASE);
  expect(Object.keys(w.writes)).toEqual([]);
  expect(w.calls.length).toBe(0);
  const s = await w.submit();
  expect(ultimo(s)).toBe(REGLAS);
});

test("submit_skip_no_adjunta", async ($, on) => {
  const w = await mundo($, on, () => out(skipOut));
  const r = await w.submit();
  expect(r.context).toEqual([]);
  expect(JSON.parse(w.writes[HB])).toEqual({ status: "skip", reason: "sin_seccion", n: 0, via: "submit" });
});

test("submit_nunca_lanza", async ($, on) => {
  const w = await mundo($, on, () => out("no soy json"));
  const r = await w.submit("texto");
  expect(r.text).toBe("texto");
  expect(r.context).toEqual([]);
  const hb = JSON.parse(w.writes[HB]);
  expect(hb.status).toBe("error");
  expect(hb.via).toBe("submit");
});

test("submit_process_run_lanza", async ($, on) => {
  const w = await mundo($, on, () => {
    throw new Error("ENOENT");
  });
  const r = await w.submit("texto");
  expect(r.text).toBe("texto");
  expect(r.context).toEqual([]);
  expect(JSON.parse(w.writes[HB]).via).toBe("submit");
});

test("submit_fs_write_denegado_entrega", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { deny: ["fs.write"] });
  const r = await w.submit();
  expect(ultimo(r)).toBe(REGLAS);
});

test("org_pista_en_hb", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { claudeJson: '{"oauthAccount":{"organizationType":"claude_team"}}' });
  await w.submit();
  expect(JSON.parse(w.writes[HB]).org).toBe("claude_team");
});

test("org_rechaza_sin_error", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { claudeJson: "rechaza" });
  await w.submit();
  expect(w.claudeJsonReads.length).toBe(1);
  const hb = JSON.parse(w.writes[HB]);
  expect(hb.org).toBe(undefined);
  expect(hb.status).toBe("ok");
});

test("org_se_lee_una_vez", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { claudeJson: '{"oauthAccount":{"organizationType":"claude_team"}}' });
  await w.submit();
  await w.submit();
  expect(w.claudeJsonReads.length).toBe(1);
  expect(JSON.parse(w.writes[HB]).org).toBe("claude_team");
});

test("org_fallo_no_se_cachea", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { claudeJson: "rechaza" });
  await w.submit();
  await w.submit();
  expect(w.claudeJsonReads.length).toBe(2);
});

test("compose_marca_store", async ($, on) => {
  const w = await mundo($, on, () => out(okOut));
  const r = await w.submit();
  expect(ultimo(r)).toBe(REGLAS);
  await w.compose();
  expect(w.store.compose).toBe(true);
  expect(JSON.parse(w.writes[HB]).via).toBe("compose");
});

test("segunda_sesion_no_duplica", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  const r = await w.submit();
  expect(r.context).toEqual([]);
  await w.compose();
  expect(w.calls.filter((c) => c.argv[1] === "rules").length).toBe(1);
});

test("turn_complete_sin_compose_desmarca_store", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  await w.submit();
  const r = await w.complete();
  expect(r.text).toBe("ok");
  expect(w.store.compose).toBe(false);
});

test("turn_complete_con_compose_no_toca", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { store: { compose: true } });
  await w.compose();
  await w.complete();
  expect(w.store.compose).toBe(true);
});

test("turn_complete_forzar_no_toca_store", async ($, on) => {
  const w = await mundo($, on, () => out(okOut), { env: { HOME: "/h", EXO_RULES_FORZAR_SUBMIT: "1" }, store: { compose: true } });
  await w.submit();
  await w.complete();
  expect(w.storeSets).toEqual([]);
});
