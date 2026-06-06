#!/usr/bin/env bash
set -euo pipefail

if [ "${#}" -lt 4 ] || [ "${#}" -gt 5 ]; then
  echo "usage: $0 <version> <sha256> <homebrew_repo_dir> <scoop_repo_dir> [branch_prefix]" >&2
  exit 1
fi

version="$1"
sha256="$2"
homebrew_repo_dir="$3"
scoop_repo_dir="$4"
branch_prefix="${5:-cxr}"
root_dir="$(cd "$(dirname "$0")/.." && pwd)"

"${root_dir}/scripts/sync-packaging.sh" "${version}" "${sha256}" "${homebrew_repo_dir}" "${scoop_repo_dir}"

update_repo() {
  local repo_dir="$1"
  local rel_path="$2"
  local branch_name="${branch_prefix}-v${version}"
  local commit_message="Update cxr to v${version}"
  local pr_title="Update cxr to v${version}"

  if [ ! -d "${repo_dir}/.git" ]; then
    echo "not a git repository: ${repo_dir}" >&2
    exit 1
  fi

  git -C "${repo_dir}" checkout -b "${branch_name}"
  git -C "${repo_dir}" add "${rel_path}"

  if git -C "${repo_dir}" diff --cached --quiet; then
    echo "no changes to commit in ${repo_dir}"
    return
  fi

  git -C "${repo_dir}" commit -m "${commit_message}"

  if command -v gh >/dev/null 2>&1; then
    (
      cd "${repo_dir}"
      gh pr create --fill --title "${pr_title}" --body "${commit_message}"
    )
  else
    echo "gh not found; created branch ${branch_name} in ${repo_dir}"
  fi
}

update_repo "${homebrew_repo_dir}" "Formula/cxr.rb"
update_repo "${scoop_repo_dir}" "bucket/cxr.json"
