// Smoke-tests for the contract-verbs verbspec surface.
// These run WITHOUT the compiled WASM (the WASM-dependent paths are exercised
// by the Rust unit tests in src/lib.rs). Here we test the verbspec contract:
// input parsing, output shape, registry projection.
import assert from "node:assert/strict";
import { toMcpToolset, toMcpTool } from "@bounded-systems/verbspec";
import { registry, validateVerb, driftVerb } from "./contract-verbs.mjs";

// Registry projects to the right verbs.
assert.deepEqual(
  Object.keys(registry).sort(),
  ["drift", "validate"],
  "registry has validate + drift"
);

// MCP toolset shape.
const toolset = toMcpToolset(registry);
assert.ok(
  toolset.some((t) => t.name === "validate"),
  "validate projects to MCP"
);
assert.ok(
  toolset.some((t) => t.name === "drift"),
  "drift projects to MCP"
);

// validate MCP tool schema has expected required fields.
const validateTool = toMcpTool(validateVerb);
assert.ok(
  !validateTool.inputSchema.required,
  "validate has no required args (all optional)"
);

// drift MCP tool schema requires value and type.
const driftTool = toMcpTool(driftVerb);
assert.ok(
  driftTool.inputSchema.required?.includes("value"),
  "drift requires value"
);
assert.ok(
  driftTool.inputSchema.required?.includes("type"),
  "drift requires type"
);

console.log(
  "✓ contract-verbs verbspec surface verified — registry + MCP schema shape"
);
