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

// Monta el mundo bajo el plugin: sesión, env, process.run, fs.write y la base de compose.
async function correr(
  $: any,
  on: any,
  run: Run,
  env: Record<string, string> = { HOME: "/h" },
  deny: readonly string[] = [],
) {
  const writes: Record<string, string> = {};
  const calls: { argv: readonly string[]; init: any }[] = [];
  mock.env(on, env);
  on("session.cwd", async () => (deny.includes("session.cwd") ? { deny: "cwd" } : { value: "/repo" }));
  on("session.id", async () => (deny.includes("session.id") ? { deny: "id" } : { value: "sid1" }));
  on("fs.write", async (_$: any, e: any) => {
    if (deny.includes("fs.write")) return { deny: "disco" };
    writes[e.path] = e.text;
    return { value: undefined };
  });
  on("process.run", async (_$: any, e: any) => {
    calls.push({ argv: e.argv, init: e.init });
    return { value: await run(e.argv, e.init) };
  });
  on("prompt.compose", async () => ({ sections: BASE }));
  const r = await $.prompt.compose({ model: "m", promptModel: "m", surfaces: [], tools: [], outputStyle: null, traits: [] });
  return { r, writes, calls };
}

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
  expect(JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"])).toEqual({ status: "skip", reason: "sin_seccion", n: 0 });
});

test("latido_ok", async ($, on) => {
  const { writes } = await correr($, on, () => out(okOut));
  expect(JSON.parse(writes["/h/.claude/exo-rules/hb-sid1"])).toEqual({ status: "ok", n: 2 });
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
