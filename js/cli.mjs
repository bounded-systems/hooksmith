#!/usr/bin/env node
// hooksmith-validate CLI — thin entry point that delegates to the verbspec registry.
import { parseArgs, runVerb } from "@bounded-systems/verbspec";
import { registry } from "./contract-verbs.mjs";

const [, , verbName, ...rest] = process.argv;
const verb = registry[verbName];

if (!verb) {
  const names = Object.keys(registry).join(", ");
  console.error(`Usage: hooksmith-validate <verb> [options]\nVerbs: ${names}`);
  process.exit(1);
}

const input = parseArgs(verb, rest);
const output = await verb.run(input);
process.stdout.write(verb.render(output));
