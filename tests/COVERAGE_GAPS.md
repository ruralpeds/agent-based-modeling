# Coverage Gaps

Functions not yet covered by tests, ranked by priority.

## Priority 1 — Critical (untested core logic)

| Module | Function / Behavior | Why |
|---|---|---|
| `src/bridge/simBridge.js` | `init()`, `start()`, `step()`, `reset()` | Worker lifecycle can't be tested in Vitest (requires real Worker + WASM); needs Playwright E2E |
| `src/worker/simWorker.js` | Full message handler (`handleCommand`) | Runs in a Worker context; not testable as unit |
| `src/renderer/canvasRenderer.js` | `render()`, `hitTest()` | Requires Canvas 2D context (jsdom does not support it) |
| `crates/…/decision_rules/reactive.rs` | `decide()` | Core behavior not directly tested — only covered indirectly via engine integration tests |
| `crates/…/decision_rules/bounded.rs` | `decide()` | Same as above |
| `crates/…/decision_rules/bdi.rs` | `decide()` | Same as above |

## Priority 2 — Important (partially covered)

| Module | Gap | Notes |
|---|---|---|
| `src/stores/paramStore.js` | `set()` clamping logic | Needs unit tests for min/max/step clamping |
| `src/stores/uiStore.js` | `selectAgent()`, `setRunning()` | Pure state mutations — straightforward to unit test |
| `src/stores/errorStore.js` | Error accumulation, size limit | `getLatest()` / `hasErrors()` with simulated dispatch events |
| `crates/…/engine/grid.rs` | `eat_at()`, `regrow()`, `new_grass()` | Grid resource dynamics not independently tested |
| `crates/…/stochastic/bayesian.rs` | `update_belief()` with each observation type | Tests exist for the overall update; individual observation branches untested |
| `crates/…/stochastic/ctmc.rs` | Rate matrix edge cases | Zero-rate rows, absorbing states |

## Priority 3 — Minor / Nice to Have

| Module | Gap |
|---|---|
| `src/utils/formatters.js` | `fmtTick` with locale-specific formatting (needs `en-US` locale fixture) |
| `src/charts/PopulationChart.js` | `_draw()` D3 rendering (requires SVG DOM) |
| `src/charts/ChartBase.js` | `resize()` observer callback |
| `crates/…/rng/mulberry32.rs` | `next_f32_signed()` range check ([-1,1)) |
| `crates/…/statistics/stats.rs` | `gini` monotonicity: more unequal distributions produce higher Gini |

## Integration Test Opportunities

- **SimEngine → SAB round-trip**: After `step()`, verify JS `Float32Array` view from `write_agent_buffer()` matches expected agent count and positions. Requires WASM build; add to Playwright suite.
- **paramStore → SimBridge → engine**: Changing params mid-run and verifying behavior change. E2E only.
- **Worker error recovery**: Intentionally passing bad JSON params and verifying `engine:error` is dispatched. E2E only.

## Performance Benchmark Candidates

| Function | Current bench? | Notes |
|---|---|---|
| `SimEngine::step()` with 5000 agents | Yes (`benches/bench.rs`) | Extend with different rule sets |
| `SpatialGrid::build()` | No | Bottleneck for large populations |
| `SpatialGrid::query_radius()` dense grid | No | Hot path in decision rules |
| `write_soa()` | No | Called every tick; SIMD-friendly |
| `gini()` over 5000 agents | No | O(n log n) sort — verify scales |
