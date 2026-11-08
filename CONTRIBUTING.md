# Contributing to Jot

Thanks for your interest! Bug reports, ideas and pull requests are welcome.

## Setup

Requirements: Node.js 24+, Rust (stable), and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```sh
npm ci
npm run tauri dev
```

## Before opening a PR

```sh
npm run check                      # type check
(cd src-tauri && cargo fmt --all && cargo clippy --all-targets && cargo test)
```

CI runs the same checks on Linux, macOS and Windows, plus an Android debug build.

## Workflow

- `main` is protected: no direct pushes, changes land via pull request only
  (CI must be green, review threads resolved). Only maintainers can merge.
- Branch from `main`, keep PRs focused, use
  [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `build:` ...).
- Link the issue your PR addresses.

## License

By contributing you agree that your contributions are licensed under the
project's [GPL-3.0 license](LICENSE).
