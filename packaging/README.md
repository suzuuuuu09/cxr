# Packaging

This directory contains templates and helpers for external package-manager repositories.

Current release strategy:

- Primary distribution: crates.io (`cargo install cxr`)
- Source release: GitHub Releases tarball
- Secondary package managers: Homebrew tap and Scoop bucket in separate repositories

## Generate package metadata

Use `./scripts/render-packaging.sh <version> <sha256>` after creating a release tarball.

Example:

```bash
./scripts/render-packaging.sh 0.1.4 deadbeef...
```

The script writes rendered files to `dist/packaging/`.

## Sync to external repositories

If you maintain separate Homebrew tap and Scoop bucket repositories, use:

```bash
./scripts/sync-packaging.sh <version> <sha256> <homebrew_repo_dir> <scoop_repo_dir>
```

The script copies:

- `dist/packaging/homebrew/cxr.rb` to `<homebrew_repo_dir>/Formula/cxr.rb`
- `dist/packaging/scoop/cxr.json` to `<scoop_repo_dir>/bucket/cxr.json`

## Publish updates

If both repositories are git checkouts and you want to commit changes and create PRs:

```bash
./scripts/publish-packaging-updates.sh <version> <sha256> <homebrew_repo_dir> <scoop_repo_dir>
```
