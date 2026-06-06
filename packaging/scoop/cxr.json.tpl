{
  "##": "Template generated from the cxr release metadata. The external bucket can adapt this to a binary or source install flow.",
  "version": "{{VERSION}}",
  "description": "A tool to generate a directory structure from a YAML template.",
  "homepage": "https://github.com/suzuuuuu09/cxr",
  "license": "MIT",
  "url": "https://github.com/suzuuuuu09/cxr/releases/download/v{{VERSION}}/cxr-{{VERSION}}.tar.gz",
  "hash": "{{SHA256}}",
  "depends": "rustup",
  "extract_dir": "cxr-{{VERSION}}",
  "installer": {
    "script": "$cargoToml = Get-ChildItem -Path $dir -Recurse -Filter Cargo.toml | Select-Object -First 1; $src = Split-Path $cargoToml.FullName -Parent; cargo install --locked --path $src --root $dir"
  },
  "bin": "bin\\cxr.exe"
}
