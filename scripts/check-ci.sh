#!/usr/bin/env bash
# ─── scripts/check-ci.sh ──────────────────────────────────────────────────────
# Inspect the latest GitHub Actions CI run for the current branch (or a named
# branch/workflow) and print a colour-coded summary of each job's status plus
# the log output from any failing steps.
#
# Usage:
#   ./scripts/check-ci.sh               # current branch, latest run
#   ./scripts/check-ci.sh --branch main # specific branch
#   ./scripts/check-ci.sh --run 12345   # specific run ID
#   ./scripts/check-ci.sh --watch       # poll every 30 s until run completes
#   ./scripts/check-ci.sh --all         # show last 10 runs for this branch
#
# Requirements: gh CLI authenticated (run `gh auth login` once)
# ──────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# ── colour helpers ────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'
ok()   { printf "${GREEN}✔  %s${RESET}\n" "$*"; }
fail() { printf "${RED}✘  %s${RESET}\n" "$*"; }
warn() { printf "${YELLOW}⚠  %s${RESET}\n" "$*"; }
info() { printf "${CYAN}ℹ  %s${RESET}\n" "$*"; }
hr()   { printf "${BOLD}%s${RESET}\n" "────────────────────────────────────────────────────────────────────────────────"; }

# ── argument parsing ──────────────────────────────────────────────────────────
BRANCH=""
RUN_ID=""
WATCH=false
SHOW_ALL=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch|-b) BRANCH="$2"; shift 2 ;;
    --run|-r)    RUN_ID="$2";  shift 2 ;;
    --watch|-w)  WATCH=true;   shift   ;;
    --all|-a)    SHOW_ALL=true; shift  ;;
    --help|-h)
      grep '^#' "$0" | sed 's/^# \{0,2\}//' | tail -n +2
      exit 0 ;;
    *) warn "Unknown option: $1"; shift ;;
  esac
done

# ── prerequisites ─────────────────────────────────────────────────────────────
if ! command -v gh &>/dev/null; then
  fail "gh CLI not found. Install from https://cli.github.com/ and run 'gh auth login'."
  exit 1
fi

if ! gh auth status &>/dev/null; then
  fail "gh CLI not authenticated. Run: gh auth login"
  exit 1
fi

# ── resolve branch ────────────────────────────────────────────────────────────
if [[ -z "$BRANCH" && -z "$RUN_ID" ]]; then
  BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")
fi

# ── helper: status icon ───────────────────────────────────────────────────────
status_icon() {
  case "$1" in
    success|completed) printf "${GREEN}✔${RESET}" ;;
    failure|failed)    printf "${RED}✘${RESET}"   ;;
    cancelled)         printf "${YELLOW}⊘${RESET}" ;;
    skipped)           printf "${CYAN}⊝${RESET}"  ;;
    in_progress|queued|waiting|pending)
                       printf "${YELLOW}⏳${RESET}" ;;
    *)                 printf "?" ;;
  esac
}

# ── show run list ─────────────────────────────────────────────────────────────
show_run_list() {
  local branch="$1"
  hr
  info "Last 10 CI runs on branch: ${BOLD}${branch}${RESET}"
  hr
  gh run list --branch "$branch" --limit 10 \
    --json databaseId,displayTitle,status,conclusion,createdAt,workflowName \
    --jq '.[] | "\(.databaseId)\t\(.conclusion // .status)\t\(.workflowName)\t\(.displayTitle[0:60])\t\(.createdAt)"' \
  | column -t -s $'\t'
}

# ── resolve run ID ────────────────────────────────────────────────────────────
resolve_run_id() {
  local branch="$1"
  gh run list --branch "$branch" --limit 1 \
    --json databaseId --jq '.[0].databaseId'
}

# ── print job summary ─────────────────────────────────────────────────────────
print_run_summary() {
  local run_id="$1"

  # Top-level run info
  local run_info
  run_info=$(gh run view "$run_id" --json \
    displayTitle,status,conclusion,createdAt,workflowName,headBranch,headSha \
    --jq '{title:.displayTitle,status:.status,conclusion:.conclusion,
           branch:.headBranch,sha:.headSha[0:8],
           workflow:.workflowName,created:.createdAt}')

  local title branch sha workflow status conclusion created
  title=$(echo "$run_info"    | jq -r '.title')
  branch=$(echo "$run_info"   | jq -r '.branch')
  sha=$(echo "$run_info"      | jq -r '.sha')
  workflow=$(echo "$run_info" | jq -r '.workflow')
  status=$(echo "$run_info"   | jq -r '.status')
  conclusion=$(echo "$run_info" | jq -r '.conclusion // .status')
  created=$(echo "$run_info"  | jq -r '.created')

  hr
  printf "${BOLD}Run #%s${RESET}  %s\n" "$run_id" "$(status_icon "$conclusion")  ${BOLD}${conclusion^^}${RESET}"
  printf "  Workflow : %s\n" "$workflow"
  printf "  Branch   : %s  (%s)\n" "$branch" "$sha"
  printf "  Title    : %s\n" "$title"
  printf "  Started  : %s\n" "$created"
  hr

  # Per-job breakdown
  local jobs
  jobs=$(gh run view "$run_id" --json jobs \
    --jq '.jobs[] | "\(.name)\t\(.conclusion // .status)\t\(.steps | map(select(.conclusion == "failure")) | length)"')

  local has_failures=false
  local failing_jobs=()

  while IFS=$'\t' read -r job_name job_conclusion fail_count; do
    local icon
    icon=$(status_icon "$job_conclusion")
    printf "  %s  %-40s  %s\n" "$icon" "$job_name" "${job_conclusion}"
    if [[ "$job_conclusion" == "failure" ]]; then
      has_failures=true
      failing_jobs+=("$job_name")
    fi
  done <<< "$jobs"

  hr

  # If there are failures, dump the failing log lines
  if $has_failures; then
    printf "\n${RED}${BOLD}Failing job logs:${RESET}\n\n"
    gh run view "$run_id" --log-failed 2>/dev/null \
      | grep -E "^(##\[|  Error|error\[|error:|warning:|FAILED|FAIL |fatal )" \
        || gh run view "$run_id" --log-failed 2>/dev/null | head -120
    hr
    printf "\n${RED}${BOLD}%d job(s) failed:${RESET} %s\n\n" \
      "${#failing_jobs[@]}" "${failing_jobs[*]}"
    return 1
  else
    ok "All jobs passed."
    return 0
  fi
}

# ── watch mode loop ───────────────────────────────────────────────────────────
watch_run() {
  local run_id="$1"
  local interval=30
  info "Watching run #${run_id} (polling every ${interval}s) … Ctrl-C to abort"
  while true; do
    local conclusion
    conclusion=$(gh run view "$run_id" --json conclusion,status \
      --jq '.conclusion // .status')
    case "$conclusion" in
      success|failure|cancelled|completed)
        break ;;
      *)
        printf "\r${YELLOW}⏳  Status: %-20s${RESET}" "$conclusion"
        sleep $interval ;;
    esac
  done
  printf "\r%80s\r" ""   # clear the progress line
}

# ── main ──────────────────────────────────────────────────────────────────────
main() {
  if $SHOW_ALL; then
    show_run_list "${BRANCH:-main}"
    exit 0
  fi

  # Resolve run ID if not given
  if [[ -z "$RUN_ID" ]]; then
    info "Fetching latest run for branch: ${BOLD}${BRANCH}${RESET}"
    RUN_ID=$(resolve_run_id "$BRANCH")
    if [[ -z "$RUN_ID" || "$RUN_ID" == "null" ]]; then
      warn "No CI runs found for branch '${BRANCH}'."
      exit 0
    fi
    info "Run ID: ${RUN_ID}"
  fi

  # Optionally wait for in-progress run to finish
  if $WATCH; then
    watch_run "$RUN_ID"
  fi

  print_run_summary "$RUN_ID"
}

main "$@"
