# Automated coverage stops at `src-tauri`; the UI is verified by hand

Status: accepted (2026-11-04)

The workspace root crate is `forgefin-ui`, a Leptos CSR (wasm) package, and it
cannot be exercised in a host test process — `web-sys`/`js-sys` compile but do
not run, and there is no headless browser loop inside Tauri. Only
`src-tauri` (`forgefin_lib`) holds business logic and SQL, and `rusqlite`
bundled makes an in-memory database cheap enough for a red-green loop.

We therefore restrict automated tests to `cargo test -p forgefin`. UI pages
under `src/pages/` get no automated coverage; they are accepted by a human
walking the acceptance-criteria checklist in `docs/spec/<feature>.md`, with
`docs/uidesign/DESIGN.md` as the visual baseline.

## Considered options

- **Automate everything** (`wasm-bindgen-test` plus a headless browser in the
  Tauri shell) — rejected: setup and maintenance cost dwarfs the value at this
  stage, and the money/voucher logic that actually breaks lives in `src-tauri`.
- **Narrow to money and voucher logic only** — rejected as too narrow: import
  parsing, company/ledger isolation and audit-log writes are equally worth
  pinning down.

## Consequences

- Test infrastructure is a prerequisite, not an option: `[dev-dependencies]`
  (`rstest`, `tempfile`) in `src-tauri/Cargo.toml` and a `Makefile` `test`
  target must land before any test-first work starts. Until then the `tdd`
  discipline is treated as unavailable.
- `docs/测试方案.md` is a strategy document plus a case list, not a coverage
  target; every case in it must be traceable to an acceptance criterion.
- Any future proposal to add UI automation has to supersede this ADR.
