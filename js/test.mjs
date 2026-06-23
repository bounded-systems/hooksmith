// Smoke-tests for both verbspec surfaces.
// These run WITHOUT compiled WASM — the WASM-dependent paths are tested by
// Rust unit tests in the component crates. Here we test verbspec contracts:
// registry shape, MCP projection, input schema correctness.
import assert from "node:assert/strict";
import { toMcpToolset, toMcpTool } from "@bounded-systems/verbspec";

// ── contract-verbs ────────────────────────────────────────────────────────────
import { registry as contractRegistry, validateVerb, driftVerb } from "./contract-verbs.mjs";

assert.deepEqual(
  Object.keys(contractRegistry).sort(),
  ["drift", "validate"],
  "contract registry has validate + drift"
);

const contractToolset = toMcpToolset(contractRegistry);
assert.ok(contractToolset.some((t) => t.name === "validate"), "validate → MCP");
assert.ok(contractToolset.some((t) => t.name === "drift"), "drift → MCP");

const validateTool = toMcpTool(validateVerb);
assert.ok(!validateTool.inputSchema.required, "validate: no required args");

const driftTool = toMcpTool(driftVerb);
assert.ok(driftTool.inputSchema.required?.includes("value"), "drift requires value");
assert.ok(driftTool.inputSchema.required?.includes("type"), "drift requires type");

// ── hook-verbs ────────────────────────────────────────────────────────────────
import {
  registry as hookRegistry,
  evaluateVerb,
  preCommitVerb,
  commitMsgVerb,
  prePushVerb,
} from "./hook-verbs.mjs";

assert.deepEqual(
  Object.keys(hookRegistry).sort(),
  ["commit-msg", "evaluate", "pre-commit", "pre-push"],
  "hook registry has evaluate + three hook verbs"
);

const hookToolset = toMcpToolset(hookRegistry);
assert.ok(hookToolset.some((t) => t.name === "evaluate"), "evaluate → MCP");
assert.ok(hookToolset.some((t) => t.name === "pre-commit"), "pre-commit → MCP");
assert.ok(hookToolset.some((t) => t.name === "commit-msg"), "commit-msg → MCP");
assert.ok(hookToolset.some((t) => t.name === "pre-push"), "pre-push → MCP");

// evaluate requires hook
const evaluateTool = toMcpTool(evaluateVerb);
assert.ok(
  evaluateTool.inputSchema.required?.includes("hook"),
  "evaluate requires hook"
);

// pre-push requires stdin
const prePushTool = toMcpTool(prePushVerb);
assert.ok(
  prePushTool.inputSchema.required?.includes("stdin"),
  "pre-push requires stdin"
);

// commit-msg requires msgFile
const commitMsgTool = toMcpTool(commitMsgVerb);
assert.ok(
  commitMsgTool.inputSchema.required?.includes("msgFile"),
  "commit-msg requires msgFile"
);

console.log(
  "✓ contract-verbs + hook-verbs verbspec surfaces verified — registry + MCP schema shape"
);
