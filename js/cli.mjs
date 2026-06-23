#!/usr/bin/env node
// Unified hooksmith CLI — delegates to the verbspec registry.
//
// Two registry surfaces:
//   contract-verbs  → validate, drift
//   hook-verbs      → evaluate, pre-commit, commit-msg, pre-push
//
// Usage (via bin aliases in package.json):
//   hooksmith-validate <verb> [options]    # contract surface
//   hooksmith-hooks    <verb> [options]    # hook engine surface
//
// Both surfaces share this entry point; the bin name selects the registry.
import { parseArgs } from "@bounded-systems/verbspec";
import { registry as contractRegistry } from "./contract-verbs.mjs";
import { registry as hookRegistry } from "./hook-verbs.mjs";

const bin = process.argv[1]?.split("/").pop() ?? "";
const registry = bin === "hooksmith-hooks" ? hookRegistry : contractRegistry;

const [, , verbName, ...rest] = process.argv;
const verb = registry[verbName];

if (!verb) {
  const names = Object.keys(registry).join(", ");
  const cmd = bin || "hooksmith-validate";
  console.error(`Usage: ${cmd} <verb> [options]\nVerbs: ${names}`);
  process.exit(1);
}

const input = parseArgs(verb, rest);
const output = await verb.run(input);
process.stdout.write(verb.render(output));
