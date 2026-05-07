# PortfolioWebsiteBuilder — Effort model

## Required context

- Normative scope: [ImplementationSpec.md](./ImplementationSpec.md)
- Widget contracts: [WidgetRegistryV1.md](./WidgetRegistryV1.md)
- Product decisions: [UpgradeQuestions.md](./UpgradeQuestions.md)
- Execution tasks: [ExecutionPlan.md](./ExecutionPlan.md)

## Sizing model

- XS = 1  
- S = 2  
- M = 3  
- L = 5  
- XL = 8  
- XXL = 13  

Risk multipliers:

- low = ×1.0  
- medium = ×1.25  
- high = ×1.6  
- very high / unknown = ×2.0  

**Final effort** (below) = rounded **size points × risk multiplier** (use for capacity planning). **[Score]** in [ExecutionPlan.md](./ExecutionPlan.md) is the baseline Fibonacci story weight before risk.

## Task effort table

| Task | Size (pts) | Risk | Final effort | Dependencies | Parallelizable |
| --- | ---: | --- | ---: | --- | --- |
| 1 Site bundle loader | 3 (M) | medium ×1.25 | 4 | — | No |
| 2 Output routing | 5 (L) | medium ×1.25 | 6 | 1 | Partly (with 3 after 1) |
| 3 Template shell | 5 (L) | medium ×1.25 | 6 | 1 | Partly (with 2 after 1) |
| 4 Merge model | 3 (M) | low ×1.0 | 3 | 3 | Partly |
| 5 Interactive CLI | 3 (M) | medium ×1.25 | 4 | 2, 3 | No |
| 6 Asset pipeline | 8 (XL) | high ×1.6 | 13 | 1 | Partly (with 2–5) |
| 7 Widget dispatcher | 5 (L) | medium ×1.25 | 6 | 3 | Partly |
| 8 Layout widgets | 5 (L) | medium ×1.25 | 6 | 7 | Partly |
| 9 Simple leaf widgets | 8 (XL) | medium ×1.25 | 10 | 7, 8 | No |
| 10 careers_tabs | 5 (L) | medium ×1.25 | 6 | 7, 9 | Partly |
| 11 apps_showcase | 13 (XXL) | high ×1.6 | 21 | 7, 6 | No |
| 12 project_grid | 5 (L) | medium ×1.25 | 6 | 7, 8 | Partly |
| 13 media_swiper (optional) | 5 (L) | medium ×1.25 | 6 | 7 | Partly |
| 14 Kometa parity bundle | 8 (XL) | medium ×1.25 | 10 | 11, 12, 10, 9 | No |
| 15 Cleanup & README | 3 (M) | low ×1.0 | 3 | 14 | No |
| 16 Rename PortfolioWebsiteBuilder + purge deprecated naming | 8 (XL) | high ×1.6 | 13 | none | Partly |

Totals (Final effort column):

- **Full path including Task 13:** ~123  
- **Reduced path excluding Task 13:** ~117  

Raw **[Score]** sum from ExecutionPlan: **91** (or **86** without Task 13).

## Critical path

Approximate longest dependency chain (by execution order, not only score):

```text
Task 1 -> Task 3 -> Task 7 -> Task 8 -> Task 9 -> Task 10 -> Task 14 -> Task 15
```

**Parallel track:** Task 6 should complete before **Task 11**; Task 2 and Task 4 can proceed alongside Task 3 once Task 1 exists.

Critical-path **raw Score** (ExecutionPlan):  
`3 + 5 + 5 + 5 + 8 + 5 + 8 + 3` = **42** (1→3→7→8→9→10→14→15), plus **Task 6** and **Task 11** on the heavy branch before 14: realistic longest weighted path includes **1→3→7→6→11→14→15** → raw **3+5+5+8+13+8+3 = 45**.

## Range and timeline

Assumptions:

- One focused agent handles ~**5** adjusted (Final effort) points per agent-day.  
- Parallelism adds integration overhead.  
- Human effort is review/approval only.

Illustrative staffing (divide **~117** Final effort points without Task 13 by throughput):

| Staffing | Likely agent-days (order-of-magnitude) |
| --- | ---: |
| 1 agent | ~23 |
| 2 parallel agents | ~13–17 |
| 3 parallel agents | ~10–14 |

## Confidence and risks

See [ExecutionPlan.md — Confidence and Risks](./ExecutionPlan.md#confidence-and-risks).
