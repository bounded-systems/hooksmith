# Git Internals to HookConcern Mapping

This document maps each `HookConcern` variant to its corresponding Git Internals chapter and explains how these concerns relate to Git's internal architecture.

## Overview

The `HookConcern` enum provides a comprehensive model of Git's internal architecture, organized by the chapters in "Git Internals" from Pro Git. Each concern represents a specific aspect of Git's object model, storage system, or operational behavior.

## Git Object Concerns (Core Objects)

These concerns map to the fundamental Git object types described in "Git Internals: Git Objects".

| Concern | Git Internals Chapter | Description | Git Path/Command |
|---------|----------------------|-------------|------------------|
| `Blob` | Git Objects | File contents stored as blobs | `.git/objects/` |
| `Tree` | Git Objects | Directory structure and file metadata | `git ls-tree` |
| `Commit` | Git Objects | Commit history and metadata | `git cat-file commit` |
| `Tag` | Git Objects | Annotated tag objects | `git cat-file tag` |
| `Ref` | Git References | Reference pointers to objects | `.git/refs/` |
| `Note` | Git References | Commit-attached metadata | `.git/refs/notes/` |
| `Attr` | Git Attributes | File-based configuration | `.gitattributes` |

## Git Reference Concerns (Detailed Ref Types)

These concerns map to the detailed reference types from "Git Internals: Git References".

| Concern | Git Internals Chapter | Description | Git Path/Command |
|---------|----------------------|-------------|------------------|
| `RefBranch` | Git References | Local branch references | `refs/heads/` |
| `RefRemote` | Git References | Remote-tracking branches | `refs/remotes/` |
| `RefTag` | Git References | Tag references | `refs/tags/` |
| `RefNote` | Git References | Note references | `refs/notes/` |
| `RefStash` | Git References | Stash references | `refs/stash` |
| `RefWorktree` | Git References | Worktree HEAD references | `worktrees/*/HEAD` |
| `RefSym` | Git References | Symbolic references | `HEAD -> refs/heads/main` |
| `HeadPointer` | Git References | Current HEAD pointer | `.git/HEAD` |
| `PackedRefs` | Git References | Packed reference storage | `.git/packed-refs` |
| `FetchHeadPointer` | Git References | Fetch result pointer | `.git/FETCH_HEAD` |
| `MergeHeadPointer` | Git References | Merge state pointer | `.git/MERGE_HEAD` |
| `CherryPickPointer` | Git References | Cherry-pick state | `.git/CHERRY_PICK_HEAD` |
| `RevertHeadPointer` | Git References | Revert state | `.git/REVERT_HEAD` |
| `OrigHead` | Git References | Previous HEAD value | `.git/ORIG_HEAD` |
| `RefLogEntry` | Git References | Reference update logs | `.git/logs/refs/` |

## Git Storage Concerns (Object Database)

These concerns map to packfile and storage concepts from "Git Internals: Packfiles".

| Concern | Git Internals Chapter | Description | Git Path/Command |
|---------|----------------------|-------------|------------------|
| `PackfileIndex` | Packfiles | Packfile index mapping | `.git/objects/pack/*.idx` |
| `PackfileData` | Packfiles | Compressed object data | `.git/objects/pack/*.pack` |
| `PackfileBitmap` | Packfiles | Bitmap index for traversal | `.git/objects/pack/*.bitmap` |
| `PackfileKeep` | Packfiles | Keep markers | `.git/objects/pack/*.keep` |
| `PackfilePromisor` | Packfiles | Partial clone promises | `.git/objects/pack/*.promisor` |
| `LooseObject` | Packfiles | Individual compressed objects | `.git/objects/??/*` |
| `ObjectDatabase` | Packfiles | Object storage container | `.git/objects/` |

## Git Local State Concerns

These concerns represent Git's working state and local constructs.

| Concern | Description | Git Path/Command |
|---------|-------------|------------------|
| `Stash` | Uncommitted work storage | `refs/stash` |
| `Worktree` | Linked working directories | `.git/worktrees/` |
| `Index` | Staging area | `.git/index` |
| `Remote` | Remote repository configs | `.git/config` |
| `Branch` | Branch-specific configs | `.git/config` |
| `Head` | Current branch reference | `.git/HEAD` |
| `Reflog` | Reference history | `.git/logs/` |

## Git Transport & Protocol Concerns

These concerns map to transport protocols from "Git Internals: Transfer Protocols".

| Concern | Git Internals Chapter | Description | Protocol |
|---------|----------------------|-------------|----------|
| `ProtocolLocal` | Transfer Protocols | Local filesystem | `/path/to/repo` |
| `ProtocolGit` | Transfer Protocols | Git protocol | `git://host/path` |
| `ProtocolHttp` | Transfer Protocols | HTTP protocol | `http://host/path` |
| `ProtocolHttps` | Transfer Protocols | HTTPS protocol | `https://host/path` |
| `ProtocolSsh` | Transfer Protocols | SSH protocol | `ssh://user@host/path` |
| `Refspec` | Git References | Remote-local ref mapping | `+refs/heads/*:refs/remotes/*` |
| `ProtocolPacket` | Transfer Protocols | Wire format packets | `GIT_TRACE_PACKET` |

## Git Runtime & Environment Concerns

These concerns map to environment variables from "Git Internals: Environment Variables".

| Concern | Git Internals Chapter | Environment Variable | Description |
|---------|----------------------|---------------------|-------------|
| `GitDirOverride` | Environment Variables | `GIT_DIR` | Override .git location |
| `WorkTreeOverride` | Environment Variables | `GIT_WORK_TREE` | Override working tree |
| `IndexFileOverride` | Environment Variables | `GIT_INDEX_FILE` | Override index path |
| `ObjectDirectoryOverride` | Environment Variables | `GIT_OBJECT_DIRECTORY` | Override objects path |
| `AlternateObjectDatabase` | Environment Variables | `GIT_ALTERNATE_OBJECT_DIRECTORIES` | Alternate storage |
| `GitConfigOverride` | Environment Variables | `GIT_CONFIG` | Override config source |
| `TraceOverride` | Environment Variables | `GIT_TRACE` | Debug tracing |
| `AuthorOverride` | Environment Variables | `GIT_AUTHOR_NAME` | Author spoofing |
| `UiOverride` | Environment Variables | `GIT_PAGER` | UI overrides |

## Git Maintenance & Recovery Concerns

These concerns map to maintenance concepts from "Git Internals: Maintenance and Data Recovery".

| Concern | Git Internals Chapter | Description | Git Command |
|---------|----------------------|-------------|-------------|
| `FsckCheck` | Maintenance | Filesystem consistency check | `git fsck` |
| `PruneOrphaned` | Maintenance | Delete unreachable objects | `git prune` |
| `RepackPackfile` | Maintenance | Combine loose objects | `git repack` |
| `GcLifecycle` | Maintenance | Garbage collection | `git gc` |
| `ReflogRepair` | Maintenance | Recover lost commits | `git reflog` |
| `IndexRecovery` | Maintenance | Reset staging area | `git reset` |

## Git Command & Operation Concerns

These concerns map to high-level Git operations and command categories.

| Concern | Git Command Category | Description | Example Commands |
|---------|---------------------|-------------|------------------|
| `Init` | Setup & Config | Repository initialization | `git init` |
| `Snapshot` | Snapshotting | File staging and commits | `git add`, `git commit` |
| `Branch` | Branching & Merging | Branch operations | `git branch`, `git checkout` |
| `Merge` | Branching & Merging | Merge operations | `git merge` |
| `Rebase` | Branching & Merging | Rebase operations | `git rebase` |
| `Push` | Sharing & Collaboration | Push operations | `git push` |
| `Pull` | Sharing & Collaboration | Pull operations | `git pull` |
| `Fetch` | Sharing & Collaboration | Fetch operations | `git fetch` |
| `Log` | Inspection & Comparison | History traversal | `git log` |
| `Diff` | Inspection & Comparison | Difference analysis | `git diff` |
| `Status` | Inspection & Comparison | Working state | `git status` |
| `Stash` | Branching & Merging | Stash operations | `git stash` |
| `Patch` | Patching | Patch operations | `git apply` |
| `Debug` | Debugging | Debug operations | `git fsck`, `git reflog` |
| `Blame` | Inspection & Comparison | Line attribution | `git blame` |
| `Plumbing` | Low-Level | Plumbing commands | `git hash-object` |
| `ObjectDb` | Low-Level | Object database | `git cat-file` |
| `Transport` | Low-Level | Transport operations | `git send-pack` |
| `ProjectInit` | Project Creation | Project setup | `git clone` |

## Git Config Concerns

These concerns represent Git configuration sections and settings.

| Concern | Config Section | Description | Example |
|---------|----------------|-------------|---------|
| `ConfigUser` | `[user]` | User identity settings | `user.name`, `user.email` |
| `ConfigCore` | `[core]` | Core repository settings | `core.autocrlf` |
| `ConfigBranch` | `[branch]` | Branch-specific settings | `branch.main.merge` |
| `ConfigRemote` | `[remote]` | Remote repository settings | `remote.origin.url` |
| `ConfigInit` | `[init]` | Initialization settings | `init.defaultBranch` |
| `ConfigColor` | `[color]` | Color output settings | `color.ui` |
| `ConfigAlias` | `[alias]` | Command aliases | `alias.st` |
| `ConfigDiff` | `[diff]` | Diff settings | `diff.algorithm` |
| `ConfigMerge` | `[merge]` | Merge settings | `merge.conflictStyle` |
| `ConfigGpg` | `[gpg]` | GPG signing settings | `gpg.program` |
| `ConfigCommit` | `[commit]` | Commit settings | `commit.template` |
| `ConfigPull` | `[pull]` | Pull settings | `pull.rebase` |
| `ConfigPush` | `[push]` | Push settings | `push.default` |
| `ConfigRebase` | `[rebase]` | Rebase settings | `rebase.autoStash` |
| `ConfigFetch` | `[fetch]` | Fetch settings | `fetch.prune` |
| `ConfigStatus` | `[status]` | Status settings | `status.showStash` |
| `ConfigTar` | `[tar]` | Archive settings | `tar.umask` |
| `ConfigRerere` | `[rerere]` | Rerere settings | `rerere.enabled` |
| `ConfigAdvice` | `[advice]` | Advice settings | `advice.detachedHead` |
| `ConfigInteractive` | `[interactive]` | Interactive settings | `interactive.singleKey` |
| `ConfigSubmodule` | `[submodule]` | Submodule settings | `submodule.recurse` |
| `ConfigFilter` | `[filter]` | Filter settings | `filter.lfs` |
| `ConfigInclude` | `[include]` | Include settings | `include.path` |
| `ConfigCredential` | `[credential]` | Credential settings | `credential.helper` |
| `ConfigHttp` | `[http]` | HTTP settings | `http.sslVerify` |
| `ConfigUrl` | `[url]` | URL settings | `url.<base>.insteadOf` |
| `ConfigSafe` | `[safe]` | Safety settings | `safe.directory` |
| `ConfigNotes` | `[notes]` | Notes settings | `notes.rewriteMode` |
| `ConfigGc` | `[gc]` | Garbage collection | `gc.auto` |
| `ConfigMaintenance` | `[maintenance]` | Maintenance settings | `maintenance.auto` |
| `ConfigPager` | `[pager]` | Pager settings | `pager.diff` |
| `ConfigWorktree` | `[worktree]` | Worktree settings | `worktree.prefix` |

## Usage Examples

### Pre-commit Hook with Blob and Tree Concerns

```jsonc
{
  "name": "pre-commit-validation",
  "scope": "git",
  "concerns": ["blob", "tree", "commit"],
  "bin": "./target/debug/hook-validate"
}
```

### Pre-push Hook with Transport Concerns

```jsonc
{
  "name": "pre-push-security",
  "scope": "git", 
  "concerns": ["push", "protocol-https", "ref-branch"],
  "bin": "./target/debug/hook-security"
}
```

### Maintenance Hook with Storage Concerns

```jsonc
{
  "name": "post-gc-validation",
  "scope": "git",
  "concerns": ["gc-lifecycle", "packfile-data", "loose-object"],
  "bin": "./target/debug/hook-maintenance"
}
```

## Schema Validation

The `HookConcern` enum is validated against the schema in `schemas/hook-concerns.schema.jsonc` to ensure:

1. **Uniqueness**: No duplicate concerns in a single hook
2. **Completeness**: All concerns are valid Git-native concepts
3. **Consistency**: Concerns align with Git's internal architecture
4. **Traceability**: Each concern maps to specific Git Internals concepts

## Future Extensions

The concern model can be extended with:

1. **Domain-specific concerns**: Language-specific file types, build artifacts
2. **CI/CD concerns**: Pipeline stages, deployment targets
3. **Security concerns**: Signing, verification, access control
4. **Performance concerns**: Caching, optimization, monitoring

Each extension maintains the Git-native foundation while adding project-specific validation capabilities.
