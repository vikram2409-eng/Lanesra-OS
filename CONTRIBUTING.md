# Contributing to Lanesra OS

Thanks for considering a contribution. This repo has two very different parts — read [Repository layout](README.md#repository-layout) first if you're not sure which one you're touching.

- **The website + browser demo** (repo root: `app.js`, `index.html`, `styles.css`, …) — a static, dependency-free single-file app. No build step.
- **The desktop edition** (`/desktop`) — a Tauri + Rust + SQLite app with a Team Workspace server mode. See [`desktop/README.md`](desktop/README.md) for the full architecture, dev setup, and a detailed "what's here / what's deferred" breakdown before you start.

## Before you start

For anything beyond a small fix, please open an issue first to discuss the approach — it saves everyone a rewritten PR. Check existing issues and the [roadmap](https://lanesraos.com/roadmap) so you're not duplicating work already in flight.

## Making a change

1. Fork the repo and create a branch off `main` (`feature/…`, `fix/…`, `docs/…` — whatever describes the change).
2. Make your change. For the desktop app, match the existing patterns in the file you're editing rather than introducing a new one — this codebase is deliberately consistent about how models/repositories/services/commands are structured.
3. **Add or update tests.** The desktop app has real integration test coverage (`core/tests/*.rs`, `server/tests/http.rs`) — a change without a test covering it will usually be asked to add one.
4. Verify before opening a PR:
   ```bash
   cd desktop
   cargo test --workspace   # Rust: core + src-tauri + server
   npm run build             # frontend: tsc + vite build
   ```
   For website changes, `node --check app.js` catches syntax errors; there's no build step to run beyond that.
5. Open a pull request against `main`. Describe what changed and why, and how you verified it (test output, screenshots for UI changes). Keep commits and PRs focused — one logical change per PR is easier to review than a bundle of unrelated ones.

## Reporting bugs / requesting features

Use the issue templates — they ask for the information that's actually needed to act on a report (steps to reproduce, expected vs. actual behavior, which part of the repo). Security vulnerabilities should **not** go through a public issue — see [SECURITY.md](SECURITY.md).

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md). Be respectful; disagreements about code are fine, personal attacks aren't.

## License

By contributing, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
