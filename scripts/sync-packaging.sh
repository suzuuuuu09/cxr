#!/usr/bin/env bash
set -euo pipefail

if [ "${#}" -ne 4 ]; then
  echo "usage: $0 <version> <sha256> <homebrew_repo_dir> <scoop_repo_dir>" >&2
  exit 1
fi

version="$1"
sha256="$2"
homebrew_repo_dir="$3"
scoop_repo_dir="$4"
root_dir="$(cd "$(dirname "$0")/.." && pwd)"

"${root_dir}/scripts/render-packaging.sh" "${version}" "${sha256}"

mkdir -p "${homebrew_repo_dir}/Formula" "${scoop_repo_dir}/bucket"

cp "${root_dir}/dist/packaging/homebrew/cxr.rb" "${homebrew_repo_dir}/Formula/cxr.rb"
cp "${root_dir}/dist/packaging/scoop/cxr.json" "${scoop_repo_dir}/bucket/cxr.json"

echo "updated Homebrew formula: ${homebrew_repo_dir}/Formula/cxr.rb"
echo "updated Scoop manifest: ${scoop_repo_dir}/bucket/cxr.json"
