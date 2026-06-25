# hooksmith

**A stream-driven Git hook engine.** A hook is a capability that reacts to a git
event stream:

```
git → hook-event → policy engine → hook-result → exit
```

Hook binaries are thin wrappers that forward the git event (args, stdin, `GIT_*`
env) to a pure policy engine and exit with its verdict. The policy engine lives
in `crates/components/hook-engine` as a host-native build today and as a
[WebAssembly component](#wasm-components) (`wasm32-wasip2`) for portable,
sandboxed, language-agnostic embedding (including JS/MCP via `jco`).

## Quick start

```bash
# Status
cargo run -p hooksmith -- status

# Evaluate a hook event (what an installed hook binary calls)
cargo run -p hooksmith -- evaluate --hook pre-commit
echo "<old> <new> refs/heads/main" | cargo run -p hooksmith -- evaluate --hook pre-push

# Reproducible (Nix)
nix run .#hooksmith -- status
```

`evaluate` exits `0` when all policies pass and non-zero when an `error`-level
finding blocks the event. Findings print to stderr as `[level] rule — message`.

## How it works

A hook binary installed in `.git/hooks/` does nothing but call back into the
engine:

```sh
#!/bin/sh
exec hooksmith evaluate --hook pre-commit "$@"
```

`hooksmith evaluate` collects the git context (trailing args, stdin, `GIT_*`
env, repo root) and hands it to `hook_engine`, which is pure — no I/O, no `git2`
— so policy logic is deterministic and testable in isolation.

### Built-in policies

| Hook | Policy | What it enforces |
|------|--------|------------------|
| `pre-commit` | object naming | Top-level tree entries follow the bounded-systems naming convention (kebab-case, known names) |
| `pre-push` | push safety | Refuses deleting a protected branch; warns when pushing to one |
| `commit-msg` | conventional-commits | **Stubbed** — emits an `info` finding; format enforcement not yet wired |

## WASM components

The reusable pieces are [WebAssembly Interface Types (WIT)] components under
`crates/components/`, built with `cargo build --target wasm32-wasip2` and
transpiled to JS/MCP via [`jco`] (see `js/`):

| Component | Role | Status |
|-----------|------|--------|
| `hook-engine` | Pure policy evaluator (the engine above) | ✅ wired |
| `contract-validator` | Validates tree entries + copy drift against declarative rules | ✅ wired |
| `git-filter` | Filters/validates git objects (blob, tree, commit, tag) | ✅ wired |
| `worktree-runner` | Worktree creation / listing | ✅ wired |
| `hook-builder` | Hook source validation + binary build/optimization | ✅ wired |
| `validation-handler` | Validation dispatch surface | ✅ wired |
| `git-proxy` | Git operation proxy | ⚠️ quarantined (WIP — excluded from the workspace) |

Build the WASM + JS surface:

```bash
cd js
npm install
npm run build      # cargo build (wasm32-wasip2) → jco transpile
npm run mcp        # serve the contract/hook verbs over MCP
```

The JS package (`@hooksmith/verbs`) exposes the engine as both a CLI
(`hooksmith-validate`, `hooksmith-hooks`) and an MCP server from one typed
[VerbSpec] contract.

## Configuration: `.hooksmith/`

The declarative policy store lives at the repo root:

```
.hooksmith/
├── schemas/        # JSON Schemas for hooks, contracts, worktrees, validation
│   ├── hooksmith/  # hook-builder, lefthook-generator, static-hook, worktree-*, …
│   ├── git/  blob/  tree/  ref/
├── git/            # agreements, contracts, scopes
└── hooks/          # git/ and github/ hook definitions
```

Contracts and agreements declare *what* must hold; the engine and
`contract-validator` enforce it.

## Bundled tools: git blob analysis (`gba`)

The repo also ships a set of git storage / hygiene auditors behind the `gba`
meta CLI. These are independent of the hook engine and useful on their own for
large monorepos and CI gates:

```bash
cargo run -p gba -- --help
cargo run -p gba -- repo-audit --fail-on 1 --format json
nix run .#gba -- rust-blob
```

| Subcommand | Answers |
|------------|---------|
| `repo-audit` | Is this repo within healthy size limits? (`--fail-on N`) |
| `rust-blob` | Which `.rs` files are biggest/hottest? |
| `delta` | Where can delta compression save space? |
| `hygiene` | What should be ignored / moved to LFS? |
| `lfs` | What should be tracked with Git LFS? |
| `packfile` | What's actually in the `.pack`? |
| `churn "<since>"` | Which files churn the most? |
| `tree-stability` | Which trees are unstable (extraction boundaries)? |
| `extract <src> <target>` | Split a directory into its own repo, with history |

Most subcommands accept `--format table|json|md`.

> Note: `gba` is currently the workspace `default-members` and the default Nix
> app (`nix run` with no attribute), a holdover from this repo's origin. Use the
> explicit `-p hooksmith` / `.#hooksmith` targets for the hook engine.

## Install & build

- **Rust**: 1.88.0 (pinned in `rust-toolchain.toml`).
- **Nix** (recommended): hermetic builds + caching.

```bash
# Dev shell with the pinned toolchain
nix develop

# Cargo
cargo build --workspace
cargo test --workspace

# Nix packages: hooksmith | gba | analysis-tools | git-hooks | dev-tools
nix build .#hooksmith
```

A `justfile` provides developer shortcuts (`just build`, `just run`, `just ci`).

## Project structure

```
hooksmith/
├── crates/
│   ├── hooksmith-app/        # `hooksmith` CLI — the hook engine host
│   ├── components/           # WASM components (hook-engine, contract-validator, …)
│   ├── gba/                  # bundled git-blob-analysis meta CLI
│   ├── git-agreement/        # contract/agreement model
│   ├── tree/ files/ snapshot/ inspector/ standalone-auditor/   # analysis crates
│   ├── xtask/                # build/codegen tasks (generates config + toolchain)
│   └── docs/                 # design notes, guides, generated docs
├── js/                       # WASM→JS/MCP surface (@hooksmith/verbs)
├── .hooksmith/               # declarative policy store (schemas, contracts, hooks)
└── flake.nix                 # Nix packages, apps, dev shell
```

## Status & roadmap

The engine and core components are wired and evaluated end-to-end; some policies
are still being filled in.

- [ ] Wire the `commit-msg` conventional-commits policy (currently stubbed)
- [ ] Un-quarantine `git-proxy` (resolve `git2` `Send`/`Sync` + async-trait issues)
- [ ] Run the engine via `wasmtime` (WASM) in the host rather than the native build
- [ ] Make `hooksmith` the workspace/Nix default once parity with `gba` tooling lands
- [ ] SARIF export for findings; richer contract-drift diagnostics

[WebAssembly Interface Types (WIT)]: https://component-model.bytecodealliance.org/design/wit.html
[`jco`]: https://github.com/bytecodealliance/jco
[VerbSpec]: https://jsr.io/@bounded-systems/verbspec
