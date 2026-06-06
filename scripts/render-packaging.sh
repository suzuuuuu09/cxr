#!/usr/bin/env bash
set -euo pipefail

if [ "${#}" -ne 2 ]; then
  echo "usage: $0 <version> <sha256>" >&2
  exit 1
fi

version="$1"
sha256="$2"
root_dir="$(cd "$(dirname "$0")/.." && pwd)"
output_dir="${root_dir}/dist/packaging"

mkdir -p "${output_dir}/homebrew" "${output_dir}/scoop"

sed \
  -e "s/{{VERSION}}/${version}/g" \
  -e "s/{{SHA256}}/${sha256}/g" \
  "${root_dir}/packaging/homebrew/cxr.rb.tpl" \
  > "${output_dir}/homebrew/cxr.rb"

sed \
  -e "s/{{VERSION}}/${version}/g" \
  -e "s/{{SHA256}}/${sha256}/g" \
  "${root_dir}/packaging/scoop/cxr.json.tpl" \
  > "${output_dir}/scoop/cxr.json"

echo "rendered packaging files to ${output_dir}"
