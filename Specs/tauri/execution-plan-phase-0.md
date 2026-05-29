# Tauri migration — Phase 0 execution plan (Preparation)

How to use this plan: each task lists **Required context** — read only those docs for that task. Cross-cutting **Confidence and Risks** below applies to every task.

**Requirements:** [requirements.md](./requirements.md) § Phase 0  
**Next phase:** [execution-plan-phase-1.md](./execution-plan-phase-1.md)

## Assumptions

- Implementation is **agent-led**; human role is approval/review.
- Go generator on `main` is still the **parity baseline** until Phase 1 completes.
- Golden snapshots are captured **once** from Go output before Rust render exists.
- `content/kometa`, `content/demo`, and `content/my-studio` are the three regression bundles.

## Confidence and Risks

**Confidence:** High.

Resolved constraints:

1. Phase scope is tooling and fixtures only — no Rust engine code required yet (except optional workspace stub).
2. Template engine choice is documented before Phase 1.1 starts.

Residual uncertainties:

1. Golden HTML may need normalization rules (whitespace, attribute order) — define them in Task 2 to avoid flaky Phase 1.4 comparisons.
2. CI workflow file may not exist yet — Task 3 may add a minimal GitHub Actions sketch only.

## Agent level legend

- `easy`: straightforward, clear requirements.
- `medium`: moderate complexity, some design decisions.
- `heavy`: complex logic, long context.

## Changelog instructions

- When a task is completed, append **`[DONE]`** to its title in this file.
- Prepend an entry at the top of [../Changelog.md](../Changelog.md) per [execution-plan.md](./execution-plan.md).

## Task breakdown

#### Task 1: Migration docs and README link [DONE] [Score:2] [Agent:easy]

**Required context**

1. [requirements.md](./requirements.md)
2. [../../README.md](../../README.md)

- Ensure [requirements.md](./requirements.md) and this index [execution-plan.md](./execution-plan.md) are linked from README under a **Migration** or **Roadmap** subsection (one short paragraph + bullet links).
- Confirm all phase execution plan files exist and are listed in the index.

**Acceptance checklist**

- README contains links to `Specs/tauri/requirements.md` and `Specs/tauri/execution-plan.md`.
- Index lists Phase 0–3 plan files with prerequisites.

Dependencies: none.

---

#### Task 2: Golden HTML baseline from Go [DONE] [Score:5] [Agent:medium]

**Required context**

1. [requirements.md](./requirements.md) §7 Testing strategy
2. [../../README.md](../../README.md) — build commands
3. Go sources: [`../../render.go`](../../render.go), [`../../cli.go`](../../cli.go)

- Run Go build for `content/kometa`, `content/demo`, `content/my-studio` into each bundle’s `output_folder`.
- Select representative outputs per bundle (at minimum: home `index.html`, one nested slug page if present, one page with heavy widgets e.g. demo `apps` or kometa home).
- Store under `crates/core/tests/golden/<bundle-id>/` (create directory layout even if `crates/` does not exist yet — or document staging path until Phase 1.1 creates workspace).
- Add `crates/core/tests/README.md` documenting **normalization rules** for comparison (e.g. trim trailing whitespace, collapse repeated newlines, optional stable attribute ordering policy).

**Acceptance checklist**

- Golden files exist for all three regression bundles.
- README in golden test folder explains how Phase 1.4 will compare Rust output to these files.
- Normalization rules are written down (not “byte-identical only” without caveats).

Dependencies: Task 1.

---

#### Task 3: Lock template engine and core crate conventions [DONE] [Score:3] [Agent:easy]

**Required context**

1. [requirements.md](./requirements.md) §3.2 Technology choices
2. [../WidgetRegistryV1.md](../WidgetRegistryV1.md) — widget partials
3. [`../../Template/layout.html`](../../Template/layout.html)

- Choose **Minijinja** or **Askama**; record decision and rationale in `crates/core/README.md` (file may be created as stub).
- Document escaping policy for URLs, CSS variables, and JSON-in-script (parity with Go `html/template` safety and [`../../render.go`](../../render.go) unsafe-marker check).
- Sketch `crates/core` public API surface: `load_site_bundle`, `validate_site_bundle`, `render_site_bundle`, `ConfigWarning` type.

**Acceptance checklist**

- `crates/core/README.md` states template engine choice and escaping rules.
- No ambiguity for Phase 1.3 implementer about which crate to use.

Dependencies: Task 2 (normalization doc can reference template output shape).

---

#### Task 4: CI sketch for Rust and cross-platform [DONE] [Score:3] [Agent:easy]

**Required context**

1. [requirements.md](./requirements.md) §7, §9
2. Existing CI if any under [`.github/workflows/`](../../.github/workflows/)

- Add or update a workflow stub (or `Specs/tauri/ci-notes.md` if CI is deferred) specifying:
  - `cargo test --workspace` on `ubuntu-latest`, `macos-latest`, `windows-latest`
  - Later: `cargo tauri build` on macOS + Windows (Phase 2)
- Document Rust toolchain version (1.77+ per requirements).

**Acceptance checklist**

- CI intent is documented in repo (workflow file or `ci-notes.md`).
- Phase 1 agents know which runners to use.

Dependencies: Task 3.

---

## Phase 0 exit criteria

- [x] Golden HTML baseline committed with normalization README.
- [x] Template engine choice locked in `crates/core/README.md`.
- [x] README links to Tauri migration specs.
- [x] CI sketch documented.

When all tasks are `[DONE]`, proceed to [execution-plan-phase-1.md](./execution-plan-phase-1.md).
