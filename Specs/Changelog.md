# Specs / implementation changelog

## PortfolioWebsiteBuilder naming — specs + Task 16

- Added **Task 16** to [ExecutionPlan.md](./ExecutionPlan.md): rename product to **PortfolioWebsiteBuilder**, purge deprecated branding (`GameDevStudio*`, `GameDev`, `sitecreator`), and phased removal of `game*` tokens in first-party code/CSS/JSON.
- [ImplementationSpec.md](./ImplementationSpec.md), [UpgradeQuestions.md](./UpgradeQuestions.md), [WidgetRegistryV1.md](./WidgetRegistryV1.md), [Effort.md](./Effort.md) updated for product naming and neutral wording.
- Go module set to **`portfoliowebsitebuilder`** in root [`go.mod`](../go.mod).

---

## 2026-02-10 — ExecutionPlan filled — specs-only

- Added concrete **15-task** breakdown to [ExecutionPlan.md](./ExecutionPlan.md) (scores, deps, required context).
- Added [Changelog.md](./Changelog.md) stub and pointed ExecutionPlan changelog steps here.
- Updated [Effort.md](./Effort.md): repo-relative links, task effort table aligned with ExecutionPlan, critical path notes.

---

Append **new entries at the top**. Suggested heading format:

`YYYY-MM-DD HH:MM TZ — <model or agent id> — <short summary>`

Body: bullet list of what changed (tasks marked `[DONE]`, behavior, docs).

---
