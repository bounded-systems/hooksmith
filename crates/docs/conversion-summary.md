
## worktree-lifecycle

- **Original**: `./worktree-lifecycle/bin/worktree-lifecycle.sh`
- **Rust Binary**: `src/bin/worktree-lifecycle.rs`
- **Type**: [0;34m[INFO][0m Analyzing: worktree-lifecycle
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
run_update_to_main {
run_detect_orphaned {
run_sync {
run_create {
run_status {
run_process {
run_create_prs {
run_auto_merge {
run_resolve_conflicts {
run_cleanup {
run_demo {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
run_update_to_main {
run_detect_orphaned {
run_sync {
run_create {
run_status {
run_process {
run_create_prs {
run_auto_merge {
run_resolve_conflicts {
run_cleanup {
run_demo {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  if ! command -v git &> /dev/null; then
  if git worktree list | grep -q "$worktree_path"; then
  if git worktree add "$worktree_path" -b "$branch_name"; then
  log_info "  git add . && git commit -m 'your message'"
  log_info "  git push -u origin $branch_name"

---

## status_report

- **Original**: `./worktree-lifecycle/scripts/status_report.sh`
- **Rust Binary**: `src/bin/status_report.rs`
- **Type**: [0;34m[INFO][0m Analyzing: status_report
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_status {
determine_state {
print_worktree_status {
generate_pr_url {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_status {
determine_state {
print_worktree_status {
generate_pr_url {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local current_branch=$(git branch --show-current)
  local status=$(git status --porcelain)
  local is_rebasing=$(git status | grep -q "rebase" && echo "true" || echo "false")
  local remote_exists=$(git ls-remote --heads origin "$current_branch" | grep -q "$current_branch" && echo "true" || echo "false")
  local is_merged=$(git branch --merged main | grep -q "$current_branch" && echo "true" || echo "false")
  local ahead_behind=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
  local behind_ahead=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
  local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
  local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)

---

## pr_creator

- **Original**: `./worktree-lifecycle/scripts/pr_creator.sh`
- **Rust Binary**: `src/bin/pr_creator.rs`
- **Type**: [0;34m[INFO][0m Analyzing: pr_creator
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
is_ready_for_pr {
push_branch {
create_pr_with_gh {
generate_pr_url {
process_ready_worktree {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
is_ready_for_pr {
push_branch {
create_pr_with_gh {
generate_pr_url {
process_ready_worktree {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local status=$(git status --porcelain)
  local behind_count=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
  local ahead_count=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
  if git push origin "$branch_name"; then
  local commit_msg=$(git log --oneline -1)
  local pr_body=$(git log --oneline main..HEAD | head -5 | sed 's/^/- /')
  local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
  local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)

---

## conflict_resolver

- **Original**: `./worktree-lifecycle/scripts/conflict_resolver.sh`
- **Rust Binary**: `src/bin/conflict_resolver.rs`
- **Type**: [0;34m[INFO][0m Analyzing: conflict_resolver
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
is_rebasing {
abort_rebase {
stash_changes {
resolve_worktree_conflicts {
push_worktree_branch {
cleanup_merged_worktree {
main {
log_info {
log_success {
log_warning {
log_error {
is_rebasing {
abort_rebase {
stash_changes {
resolve_worktree_conflicts {
push_worktree_branch {
cleanup_merged_worktree {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  git status --porcelain | grep -q "^UU\|^AA\|^DD" || git status | grep -q "rebase in progress"
  git rebase --abort
  if ! git diff --quiet; then
  git stash push -m "Auto-stash during conflict resolution $(date)"
  local status=$(git status --porcelain)
  log_info "Current status: $(git status --short)"
  git rebase --abort
  if ! git diff --quiet; then
  if git rebase main; then
  git rebase --abort

---

## state_machine

- **Original**: `./worktree-lifecycle/lib/state_machine.sh`
- **Rust Binary**: `src/bin/state_machine.rs`
- **Type**: [0;34m[INFO][0m Analyzing: state_machine
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_state {
transition_state {
generate_pr_url {
process_worktree {
print_diagram {
process_all_worktrees {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_state {
transition_state {
generate_pr_url {
process_worktree {
print_diagram {
process_all_worktrees {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local current_branch=$(git branch --show-current)
  local status=$(git status --porcelain)
  local is_rebasing=$(git status | grep -q "rebase" && echo "true" || echo "false")
  local is_merged=$(git branch --merged main | grep -q "$current_branch" && echo "true" || echo "false")
  local ahead_behind=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
  local behind_ahead=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
  if git rebase main; then
  git rebase --abort
  if git push origin "$branch_name"; then
  git worktree remove "$worktree_path" --force

---

## run_architecture_demo

- **Original**: `./examples/run_architecture_demo.sh`
- **Rust Binary**: `src/bin/run_architecture_demo.rs`
- **Type**: [0;34m[INFO][0m Analyzing: run_architecture_demo
general_utility
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: print_step {
print_success {
print_warning {
print_error {
print_info {
print_step {
print_success {
print_warning {
print_error {
print_info {

### Git Commands
[0;34m[INFO][0m No git commands found

---

## worktree-state-machine

- **Original**: `./scripts/worktree-state-machine.sh`
- **Rust Binary**: `src/bin/worktree-state-machine.rs`
- **Type**: [0;34m[INFO][0m Analyzing: worktree-state-machine
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_state {
transition_state {
generate_pr_url {
process_worktree {
print_diagram {
process_all_worktrees {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_state {
transition_state {
generate_pr_url {
process_worktree {
print_diagram {
process_all_worktrees {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local current_branch=$(git branch --show-current)
  local status=$(git status --porcelain)
  local is_rebasing=$(git status | grep -q "rebase" && echo "true" || echo "false")
  local is_merged=$(git branch --merged main | grep -q "$current_branch" && echo "true" || echo "false")
  local ahead_behind=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
  local behind_ahead=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
  if git rebase main; then
  git rebase --abort
  if git push origin "$branch_name"; then
  git worktree remove "$worktree_path" --force

---

## create-worktree-pr

- **Original**: `./scripts/create-worktree-pr.sh`
- **Rust Binary**: `src/bin/create-worktree-pr.rs`
- **Type**: [0;34m[INFO][0m Analyzing: create-worktree-pr
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
is_ready_for_pr {
push_branch {
create_pr_with_gh {
generate_pr_url {
process_ready_worktree {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
is_ready_for_pr {
push_branch {
create_pr_with_gh {
generate_pr_url {
process_ready_worktree {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local status=$(git status --porcelain)
  local behind_count=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
  local ahead_count=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
  if git push origin "$branch_name"; then
  local commit_msg=$(git log --oneline -1)
  local pr_body=$(git log --oneline main..HEAD | head -5 | sed 's/^/- /')
  local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
  local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)

---

## verify-worktree-1to1

- **Original**: `./scripts/verify-worktree-1to1.sh`
- **Rust Binary**: `src/bin/verify-worktree-1to1.rs`
- **Type**: [0;34m[INFO][0m Analyzing: verify-worktree-1to1
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
get_remote_branches {
get_worktree_branches {
worktree_exists {
remote_branch_exists {
verify_worktree_sync {
show_status {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
get_remote_branches {
get_worktree_branches {
worktree_exists {
remote_branch_exists {
verify_worktree_sync {
show_status {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  git branch -r | grep "origin/" | grep -v "origin/main" | grep -v "origin/HEAD" | sed 's/origin\///' | sort
  git worktree list | grep -v "main" | sed -n 's/.*\[\([^]]*\)\]/\1/p' | sort
  git worktree list | grep -q "\[$branch_name\]"
  git ls-remote --heads origin "$branch_name" | grep -q "$branch_name"
  git fetch --all --prune > /dev/null 2>&1
  git worktree list
  git branch -r | grep "origin/" | grep -v "origin/main" | grep -v "origin/HEAD" | sort

---

## sync-all-remote-branches

- **Original**: `./scripts/sync-all-remote-branches.sh`
- **Rust Binary**: `src/bin/sync-all-remote-branches.rs`
- **Type**: [0;34m[INFO][0m Analyzing: sync-all-remote-branches
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
fetch_remote_branches {
get_remote_branches {
worktree_exists {
create_worktree {
sync_main_branch {
sync_all_branches {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
fetch_remote_branches {
get_remote_branches {
worktree_exists {
create_worktree {
sync_main_branch {
sync_all_branches {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  if ! command -v git &> /dev/null; then
  git fetch --all --prune
  done < <(git branch -r | grep "origin/" | sort)
  if git worktree list | grep -q "$worktree_path"; then
  git worktree remove "$worktree_path" 2>/dev/null || true
  git branch -D "$branch_name" 2>/dev/null || true
  if git show-ref --verify --quiet refs/heads/"$branch_name"; then
  if git worktree add "$worktree_path" "$branch_name"; then
  if git worktree add "$worktree_path" -b "$branch_name" "origin/$branch_name"; then
  git fetch origin main

---

## update-worktrees-to-main

- **Original**: `./scripts/update-worktrees-to-main.sh`
- **Rust Binary**: `src/bin/update-worktrees-to-main.rs`
- **Type**: [0;34m[INFO][0m Analyzing: update-worktrees-to-main
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
update_worktree_to_main {
process_all_worktrees {
show_status {
show_usage {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
update_worktree_to_main {
process_all_worktrees {
show_status {
show_usage {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local commits_behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
  if git branch --merged origin/main | grep -q "$branch_name"; then
  git worktree remove "$worktree_path" 2>/dev/null || true
  git branch -D "$branch_name" 2>/dev/null || true
  if git rebase origin/main; then
  git worktree remove "$worktree_path" 2>/dev/null || true
  git branch -D "$branch_name" 2>/dev/null || true
  git worktree add "$worktree_path" -b "$branch_name"
  local worktrees=$(git worktree list --porcelain | grep "workdir" | awk '{print $2}')
  git worktree list

---

## cleanup-old-worktrees

- **Original**: `./scripts/cleanup-old-worktrees.sh`
- **Rust Binary**: `src/bin/cleanup-old-worktrees.rs`
- **Type**: [0;34m[INFO][0m Analyzing: cleanup-old-worktrees
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: print_status {
remove_worktree {
create_pr_for_ready {
main {
print_status {
remove_worktree {
create_pr_for_ready {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  if git status | grep -q "rebase"; then
  git rebase --abort 2>/dev/null || true
  local branch=$(git branch --show-current)
  git worktree remove "$worktree_name" --force 2>/dev/null || {
  git branch -D "$branch" 2>/dev/null || true
  local branch=$(git branch --show-current)
  if git ls-remote --heads origin "$branch" | grep -q "$branch"; then
  local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')

---

## auto-merge-all-prs

- **Original**: `./scripts/auto-merge-all-prs.sh`
- **Rust Binary**: `src/bin/auto-merge-all-prs.rs`
- **Type**: [0;34m[INFO][0m Analyzing: auto-merge-all-prs
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
get_worktree_branches {
branch_has_open_pr {
get_pr_number {
merge_pr {
auto_merge_all_prs {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
get_worktree_branches {
branch_has_open_pr {
get_pr_number {
merge_pr {
auto_merge_all_prs {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  if ! command -v git &> /dev/null; then
  done < <(git worktree list)

---

## comprehensive-worktree-workflow

- **Original**: `./scripts/comprehensive-worktree-workflow.sh`
- **Rust Binary**: `src/bin/comprehensive-worktree-workflow.rs`
- **Type**: [0;34m[INFO][0m Analyzing: comprehensive-worktree-workflow
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
create_demo_worktree {
demonstrate_workflow {
show_summary {
show_usage {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
create_demo_worktree {
demonstrate_workflow {
show_summary {
show_usage {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  git worktree add "$worktree_path" -b "$branch_name"
  git add demo.md
  git commit -m "feat: add demo content for $branch_name"

---

## safe-worktree-cleanup

- **Original**: `./scripts/safe-worktree-cleanup.sh`
- **Rust Binary**: `src/bin/safe-worktree-cleanup.rs`
- **Type**: [0;34m[INFO][0m Analyzing: safe-worktree-cleanup
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m No functions found in script

### Git Commands
[0;34m[INFO][0m Found git commands:
  worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
  git worktree remove "$worktree_path" --force
  status=$(git status --porcelain)
  git worktree remove "$worktree_path" --force

---

## update-worktrees

- **Original**: `./scripts/update-worktrees.sh`
- **Rust Binary**: `src/bin/update-worktrees.rs`
- **Type**: [0;34m[INFO][0m Analyzing: update-worktrees
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m No functions found in script

### Git Commands
[0;34m[INFO][0m Found git commands:
  git fetch origin
  worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2 | grep -v "^$(pwd)$")
  branch=$(cd "$worktree" && git branch --show-current)
  if [ -n "$(git status --porcelain)" ]; then
  git status --short
  current_commit=$(git rev-parse HEAD)
  echo "   Current commit: $(git log --oneline -1)"
  behind_count=$(git rev-list --count HEAD..origin/main)
  if git rebase origin/main; then
  echo "   New commit: $(git log --oneline -1)"

---

## update-all-worktrees-to-main

- **Original**: `./scripts/update-all-worktrees-to-main.sh`
- **Rust Binary**: `src/bin/update-all-worktrees-to-main.rs`
- **Type**: [0;34m[INFO][0m Analyzing: update-all-worktrees-to-main
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
get_worktree_branches {
update_worktree_to_main {
create_pr_for_worktree {
update_all_worktrees {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
get_worktree_branches {
update_worktree_to_main {
create_pr_for_worktree {
update_all_worktrees {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  if ! command -v git &> /dev/null; then
  done < <(git worktree list)
  git fetch origin main
  behind_count=$(git rev-list HEAD..origin/main --count)
  if git reset --hard origin/main; then
  if git push --force-with-lease origin "$branch_name"; then
  worktree_path=$(git worktree list | grep "\[$branch\]" | awk '{print $1}')

---

## git-worktree-wrapper

- **Original**: `./scripts/git-worktree-wrapper.sh`
- **Rust Binary**: `src/bin/git-worktree-wrapper.rs`
- **Type**: [0;34m[INFO][0m Analyzing: git-worktree-wrapper
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: show_worktree_guidance {
show_worktree_status {
main {
show_worktree_guidance {
show_worktree_status {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  echo -e "${RED}❌ Please use worktree commands instead of git worktree:${NC}"
  echo -e "${YELLOW}  📁 Or use ${GREEN}git xworktree${YELLOW} for direct git worktree access${NC}"
  echo -e "${CYAN}  git wtl${NC}  - List worktrees"
  echo -e "${CYAN}  git wtc${NC}  - Create worktree"
  echo -e "${CYAN}  git wts${NC}  - Switch worktree"
  echo -e "${CYAN}  git wtr${NC}  - Remove worktree"
  echo -e "${YELLOW}🔧 To execute the original git worktree command, use:${NC}"
  echo -e "${GREEN}git xworktree $*${NC}"
  echo -e "${GREEN}git wtl${NC}  - List worktrees"
  echo -e "${GREEN}git wtc${NC}  - Create worktree"

---

## ensure-clean-main

- **Original**: `./scripts/ensure-clean-main.sh`
- **Rust Binary**: `src/bin/ensure-clean-main.rs`
- **Type**: [0;34m[INFO][0m Analyzing: ensure-clean-main
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m No functions found in script

### Git Commands
[0;34m[INFO][0m No git commands found

---

## worktree-status-report

- **Original**: `./scripts/worktree-status-report.sh`
- **Rust Binary**: `src/bin/worktree-status-report.rs`
- **Type**: [0;34m[INFO][0m Analyzing: worktree-status-report
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_status {
determine_state {
print_worktree_status {
generate_pr_url {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
get_worktree_status {
determine_state {
print_worktree_status {
generate_pr_url {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local current_branch=$(git branch --show-current)
  local status=$(git status --porcelain)
  local is_rebasing=$(git status | grep -q "rebase" && echo "true" || echo "false")
  local remote_exists=$(git ls-remote --heads origin "$current_branch" | grep -q "$current_branch" && echo "true" || echo "false")
  local is_merged=$(git branch --merged main | grep -q "$current_branch" && echo "true" || echo "false")
  local ahead_behind=$(git rev-list --count main..HEAD 2>/dev/null || echo "0")
  local behind_ahead=$(git rev-list --count HEAD..main 2>/dev/null || echo "0")
  local repo_url=$(git config --get remote.origin.url | sed 's/\.git$//')
  local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)

---

## build_xtask

- **Original**: `./scripts/build_xtask.sh`
- **Rust Binary**: `src/bin/build_xtask.rs`
- **Type**: [0;34m[INFO][0m Analyzing: build_xtask
build_script
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m No functions found in script

### Git Commands
[0;34m[INFO][0m No git commands found

---

## migrate-worktrees-to-wt

- **Original**: `./scripts/migrate-worktrees-to-wt.sh`
- **Rust Binary**: `src/bin/migrate-worktrees-to-wt.rs`
- **Type**: [0;34m[INFO][0m Analyzing: migrate-worktrees-to-wt
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
is_worktree {
get_worktree_branch {
move_worktree {
handle_external_worktree {
migrate_worktrees {
cleanup_old_directories {
main {
log_info {
log_success {
log_warning {
log_error {
is_worktree {
get_worktree_branch {
move_worktree {
handle_external_worktree {
migrate_worktrees {
cleanup_old_directories {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  # Function to check if a directory is a git worktree
  # Try to get branch from git worktree list
  branch_name=$(git worktree list --porcelain | grep "^worktree $worktree_path" -A 1 | grep "^branch" | cut -d' ' -f2 | sed 's|refs/heads/||')
  # Fallback: try to get branch from .git file content
  if git worktree remove "$worktree_path" 2>/dev/null; then
  if git worktree add "$new_path" "$branch_name" 2>/dev/null; then
  local worktrees=$(git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2)
  local repo_root=$(git rev-parse --show-toplevel)
  # Check if we're in a git repository
  if ! git rev-parse --git-dir > /dev/null 2>&1; then

---

## build_xtask_cross

- **Original**: `./scripts/build_xtask_cross.sh`
- **Rust Binary**: `src/bin/build_xtask_cross.rs`
- **Type**: [0;34m[INFO][0m Analyzing: build_xtask_cross
build_script
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m No functions found in script

### Git Commands
[0;34m[INFO][0m No git commands found

---

## intelligent-worktree-cleanup

- **Original**: `./scripts/intelligent-worktree-cleanup.sh`
- **Rust Binary**: `src/bin/intelligent-worktree-cleanup.rs`
- **Type**: [0;34m[INFO][0m Analyzing: intelligent-worktree-cleanup
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: print_status {
analyze_worktree {
execute_decision {
create_prs_for_ready {
main {
print_status {
analyze_worktree {
execute_decision {
create_prs_for_ready {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  local branch=$(git branch --show-current)
  local commit_count=$(git log --oneline --since="1 week ago" | wc -l)
  local last_commit=$(git log --oneline -1)
  local conflicts=$(git diff --name-only --diff-filter=U 2>/dev/null || true)
  local rebase_status=$(git status | grep "rebase" || true)
  local behind_count=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo "0")
  if git ls-remote --heads origin "$branch" | grep -q "$branch"; then
  if git branch --merged origin/main | grep -q "$branch"; then
  local branch=$(git branch --show-current)
  if git status | grep -q "rebase"; then

---

## detect-orphaned-branches

- **Original**: `./scripts/detect-orphaned-branches.sh`
- **Rust Binary**: `src/bin/detect-orphaned-branches.rs`
- **Type**: [0;34m[INFO][0m Analyzing: detect-orphaned-branches
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
get_worktree_branches {
get_local_branches {
find_orphaned_branches {
create_worktree_for_branch {
delete_orphaned_branch {
handle_orphaned_branches {
main {
log_info {
log_success {
log_warning {
log_error {
log_header {
show_usage {
check_dependencies {
get_worktree_branches {
get_local_branches {
find_orphaned_branches {
create_worktree_for_branch {
delete_orphaned_branch {
handle_orphaned_branches {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  if ! command -v git &> /dev/null; then
  done < <(git worktree list)
  done < <(git branch --list)
  if git worktree list | grep -q "$worktree_path"; then
  if git worktree add "$worktree_path" "$branch_name"; then
  if git branch --merged main | grep -q "^[[:space:]]*$branch_name$"; then
  if git branch -d "$branch_name"; then
  if git branch -D "$branch_name"; then

---

## resolve-worktree-conflicts

- **Original**: `./scripts/resolve-worktree-conflicts.sh`
- **Rust Binary**: `src/bin/resolve-worktree-conflicts.rs`
- **Type**: [0;34m[INFO][0m Analyzing: resolve-worktree-conflicts
worktree_management
- **Status**: Converted (basic structure)
- **TODO**: Implement specific functionality

### Key Functions
[0;34m[INFO][0m Found functions: log_info {
log_success {
log_warning {
log_error {
is_rebasing {
abort_rebase {
stash_changes {
resolve_worktree_conflicts {
push_worktree_branch {
cleanup_merged_worktree {
main {
log_info {
log_success {
log_warning {
log_error {
is_rebasing {
abort_rebase {
stash_changes {
resolve_worktree_conflicts {
push_worktree_branch {
cleanup_merged_worktree {
main {

### Git Commands
[0;34m[INFO][0m Found git commands:
  git status --porcelain | grep -q "^UU\|^AA\|^DD" || git status | grep -q "rebase in progress"
  git rebase --abort
  if ! git diff --quiet; then
  git stash push -m "Auto-stash during conflict resolution $(date)"
  local status=$(git status --porcelain)
  log_info "Current status: $(git status --short)"
  git rebase --abort
  if ! git diff --quiet; then
  if git rebase main; then
  git rebase --abort

---
