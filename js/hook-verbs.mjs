// VerbSpec surface for the hook-engine WASM component.
//
// Two verbs:
//   evaluate  — run a hook event through the policy engine, return findings
//   pre-commit / pre-push / commit-msg — thin alias verbs for each hook kind
//
// Build the WASM and its JS bindings first:
//   npm run build:hook-engine   (cargo → wasm32-wasip2 → jco transpile)
//
// The jco-transpiled module lives at dist/hook-engine/hook_engine.js.
// At runtime: no Rust toolchain needed — just the .wasm binary and Node 18+.
import { defineVerb } from "@bounded-systems/verbspec";
import { execFileSync, spawnSync } from "node:child_process";
import { z } from "zod";

// ── WASM loader ───────────────────────────────────────────────────────────────

let _wasm = null;

async function wasm() {
  if (_wasm) return _wasm;
  try {
    _wasm = await import("./dist/hook-engine/hook_engine.js");
  } catch {
    throw new Error(
      "hook-engine WASM not built — run `npm run build:hook-engine` in js/ first"
    );
  }
  return _wasm;
}

// ── Shared types ──────────────────────────────────────────────────────────────

const Level = z.enum(["error", "warn", "info"]);

const Finding = z.object({
  level: Level,
  rule: z.string(),
  message: z.string(),
  suggestion: z.string().nullable(),
  path: z.string().nullable(),
});

const HookResult = z.object({
  allow: z.boolean(),
  exitCode: z.number().int().min(0).max(255),
  summary: z.string(),
  findings: z.array(Finding),
});

// ── evaluate verb ─────────────────────────────────────────────────────────────
//
// General-purpose verb: supply any hook-kind and get back the policy verdict.
// Hook binaries call this; CI tools call it via MCP.

export const evaluateVerb = defineVerb({
  id: "evaluate",
  summary: "Evaluate a git hook event through the stream-driven policy engine.",
  actor: "evaluate",
  input: z.object({
    hook: z
      .string()
      .describe(
        'Git hook name that fired (e.g. "pre-commit", "commit-msg", "pre-push").'
      ),
    args: z
      .array(z.string())
      .optional()
      .describe("Arguments git passed to the hook (e.g. commit-msg file path)."),
    stdin: z
      .string()
      .optional()
      .describe(
        "Stdin content for hooks that receive it (pre-push, pre-receive, post-receive)."
      ),
    repo: z
      .string()
      .optional()
      .describe("Absolute path to the repository root (default: cwd)."),
  }),
  output: HookResult,
  run: async (input) => {
    const { evaluate } = await wasm();

    // Map hook name string → WIT enum discriminant.
    const kindMap = {
      "pre-commit": "pre-commit",
      "commit-msg": "commit-msg",
      "post-commit": "post-commit",
      "pre-push": "pre-push",
      "pre-merge-commit": "pre-merge-commit",
      "pre-rebase": "pre-rebase",
      "post-checkout": "post-checkout",
      "post-merge": "post-merge",
      "post-rewrite": "post-rewrite",
      "pre-receive": "pre-receive",
      "update": "update",
      "post-receive": "post-receive",
      "post-update": "post-update",
      "applypatch-msg": "applypatch-msg",
      "pre-applypatch": "pre-applypatch",
      "post-applypatch": "post-applypatch",
      "process-filter": "process-filter",
      "fsmonitor-watchman": "fsmonitor-watchman",
    };

    const repo = input.repo ?? process.cwd();

    // Collect GIT_* env vars as list<tuple<string,string>>.
    const env = Object.entries(process.env)
      .filter(([k]) => k.startsWith("GIT_") || k === "HOME")
      .map(([k, v]) => [k, v ?? ""]);

    const result = evaluate({
      kind: kindMap[input.hook] ?? "unknown",
      args: input.args ?? [],
      stdin: input.stdin ?? null,
      env,
      repoRoot: repo,
    });

    return {
      allow: result.allow,
      exitCode: result.exitCode,
      summary: result.summary,
      findings: result.findings.map((f) => ({
        level: f.level,
        rule: f.rule,
        message: f.message,
        suggestion: f.suggestion ?? null,
        path: f.path ?? null,
      })),
    };
  },
  render: (out) => {
    const glyph = { error: "✗", warn: "⚠", info: "·" };
    const verdict = out.allow ? "✓ allowed" : "✗ blocked";
    const lines = [
      `\n  ${verdict} — ${out.summary}`,
      `  ${"─".repeat(54)}`,
    ];
    if (out.findings.length === 0) {
      lines.push("  (no findings)");
    } else {
      for (const f of out.findings) {
        const loc = f.path ? ` [${f.path}]` : "";
        const hint = f.suggestion ? `\n       hint: ${f.suggestion}` : "";
        lines.push(
          `  ${glyph[f.level] ?? "·"} ${f.rule}${loc}: ${f.message}${hint}`
        );
      }
    }
    lines.push("");
    return lines.map((l) => l + "\n").join("");
  },
});

// ── pre-commit verb ───────────────────────────────────────────────────────────
//
// Thin alias: reads the git tree and passes it to the evaluate verb.
// The pre-commit hook binary calls: hooksmith-hooks pre-commit

export const preCommitVerb = defineVerb({
  id: "pre-commit",
  summary: "Run pre-commit policies (naming, lint stubs) via the hook engine.",
  actor: "evaluate",
  input: z.object({
    repo: z.string().optional().describe("Repository root (default: cwd)."),
  }),
  output: HookResult,
  run: async (input) => {
    const repo = input.repo ?? process.cwd();

    // Collect tree entries and pass via env so the WASM component can see them.
    let treeEntries = "";
    try {
      treeEntries = execFileSync("git", ["-C", repo, "ls-tree", "--name-only", "HEAD"], {
        encoding: "utf8",
      }).trim();
    } catch {
      // Fresh repo with no commits — empty tree is fine.
    }

    // Temporarily set env var so the naming policy can read it.
    const savedEnv = process.env.HOOKSMITH_TREE_ENTRIES;
    process.env.HOOKSMITH_TREE_ENTRIES = treeEntries;

    try {
      return await evaluateVerb.run({ hook: "pre-commit", repo });
    } finally {
      if (savedEnv === undefined) {
        delete process.env.HOOKSMITH_TREE_ENTRIES;
      } else {
        process.env.HOOKSMITH_TREE_ENTRIES = savedEnv;
      }
    }
  },
  render: evaluateVerb.render,
});

// ── commit-msg verb ───────────────────────────────────────────────────────────

export const commitMsgVerb = defineVerb({
  id: "commit-msg",
  summary: "Run commit-msg policies (Conventional Commits) via the hook engine.",
  actor: "evaluate",
  input: z.object({
    msgFile: z
      .string()
      .describe("Path to the commit message file (first arg git passes)."),
    repo: z.string().optional(),
  }),
  output: HookResult,
  run: async (input) => {
    const { readFileSync } = await import("node:fs");
    let stdin;
    try {
      stdin = readFileSync(input.msgFile, "utf8");
    } catch {
      stdin = undefined;
    }
    return evaluateVerb.run({
      hook: "commit-msg",
      args: [input.msgFile],
      stdin,
      repo: input.repo,
    });
  },
  render: evaluateVerb.render,
});

// ── pre-push verb ─────────────────────────────────────────────────────────────

export const prePushVerb = defineVerb({
  id: "pre-push",
  summary:
    "Run pre-push policies (protect main/master from deletion) via the hook engine.",
  actor: "evaluate",
  input: z.object({
    stdin: z
      .string()
      .describe(
        "Stdin content git passes to pre-push (ref lines: local-ref local-sha remote-ref remote-sha)."
      ),
    repo: z.string().optional(),
  }),
  output: HookResult,
  run: async (input) =>
    evaluateVerb.run({ hook: "pre-push", stdin: input.stdin, repo: input.repo }),
  render: evaluateVerb.render,
});

// ── registry ──────────────────────────────────────────────────────────────────

export const registry = {
  evaluate: evaluateVerb,
  "pre-commit": preCommitVerb,
  "commit-msg": commitMsgVerb,
  "pre-push": prePushVerb,
};
