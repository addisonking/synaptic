# releasing synaptic

## how to release

run the local release script to bump the version and trigger the build:

```bash
bun run release patch   # 0.1.0 -> 0.1.1
bun run release minor   # 0.1.0 -> 0.2.0
bun run release major   # 0.1.0 -> 1.0.0
```

this script will:
1. bump the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
2. commit the changes
3. create an annotated git tag (e.g., `v0.2.0`)
4. push the commit and tag to origin

## what happens next

pushing a tag matching `v*.*.*` automatically triggers the **release** github actions workflow (`.github/workflows/release.yml`).

it runs on `macos-latest` (apple silicon) and:
1. installs bun and rust
2. installs frontend dependencies
3. builds the tauri app for `aarch64-apple-darwin`
4. creates a **draft** github release with the `.dmg` and `.app` bundle attached

you then manually review and publish the draft release in the github ui.

## notes

- no code signing is configured
- only macos arm64 builds are produced
- make sure your working directory is clean before running `bun run release`
- the script exits with an error if there are uncommitted changes
