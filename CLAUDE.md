# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Identity
- **Name:** abm-wasm-simulator
- **Computation:** Rust (edition 2021) → WebAssembly via wasm-pack
- **Visualization:** Pure D3.js v7 — NO React, NO Vue, NO Angular
- **JS/WASM Bridge:** wasm-bindgen + SharedArrayBuffer (zero-copy hot path)
- **Build:** Cargo + wasm-pack + Vite 5 + pnpm
- **Tests:** `cargo test` (Rust) | Vitest (JS integration) | Playwright (E2E)
- **Rust toolchain:** stable + wasm32-unknown-unknown target
- **Node:** >=20.0.0

## Commands

```bash
# Build WASM (run whenever Rust changes):
wasm-pack build crates/sim-engine --target web --out-dir public/pkg --release

# Full build (Rust → WASM → Vite bundle):
pnpm build

# Dev server (watches both Rust and JS):
pnpm dev

# Testing:
cargo test -p sim-engine          # Rust unit tests (no WASM target needed)
cargo bench -p sim-engine         # Criterion benchmarks
pnpm test                         # Vitest JS + WASM integration tests
pnpm test:e2e                     # Playwright E2E tests
pnpm ci                           # All of the above + lint + build

# Lint/format:
cargo clippy -p sim-engine        # Must pass (deny(warnings) in CI)
cargo fmt -p sim-engine           # Auto-format Rust
```

## Architecture: The Two-Layer Contract

**Layer 1 — Rust/WASM (owns computation):** All simulation logic lives in `crates/sim-engine/`. Compiles to WebAssembly, runs inside a Web Worker.

**Layer 2 — D3.js (owns presentation):** All rendering and UI in `src/`. Reads agent state from SharedArrayBuffer. Never calls WASM directly — only through `src/bridge/simBridge.js`.

**Data flow (one direction only):**
```
WASM engine → SharedArrayBuffer → JS renderer/charts
JS UI → d3-dispatch → Worker postMessage → WASM
```

### Module Responsibilities — Never Violate

| Path | Responsibility | Banned |
|---|---|---|
| `crates/sim-engine/src/lib.rs` | Pure Rust simulation logic | Any wasm-bindgen, browser APIs |
| `crates/sim-engine/src/wasm_api.rs` | Only place with `#[wasm_bindgen]` annotations | Business logic |
| `src/bridge/simBridge.js` | Only JS file that calls WASM functions | Direct DOM manipulation |
| `src/renderer/` | Canvas 2D drawing from Float32Array | D3 imports, WASM imports |
| `src/charts/` | D3 SVG charts | Canvas API, WASM imports |
| `src/stores/` | Plain JS objects + d3-dispatch state | Framework state |
| `src/worker/` | WASM runtime | Main-thread APIs |

### SharedArrayBuffer Memory Layout (SoA format)

All agent data is stored as Structure of Arrays in a SharedArrayBuffer. Field offsets are defined in `src/bridge/bufferLayout.js`. The canonical layout (10 × f32 per agent, MAX_AGENTS = 5000):

```
Offset    Field         Range
──────────────────────────────────────
0 * N     pos_x         [0.0, GRID_W)
1 * N     pos_y         [0.0, GRID_H)
2 * N     energy        [0.0, MAX_ENERGY]
3 * N     agent_type    0=prey, 1=predator
4 * N     action        ACTION_CODES
5 * N     age           [0.0, ∞)
6 * N     intention_id  INTENTION_CODES
7 * N     kills         [0.0, ∞)
8 * N     offspring     [0.0, ∞)
9 * N     agent_id      unique integer cast to f32

Header (bytes 0–31, Int32Array):
  [0] agent_count (Atomics)
  [1] tick        (Atomics)
  [2] write_lock  (Atomics.compareExchange)
```

**CRITICAL:** Never change the SAB layout without updating both the Rust writer (`write_agent_buffer`) and JS reader (`bufferLayout.js`) simultaneously.

### Rust Crate Structure

```
crates/sim-engine/src/
├── lib.rs                  Crate root — pub mod declarations only
├── wasm_api.rs             #[wasm_bindgen] thin wrappers (WasmSimEngine)
├── engine/
│   ├── sim_engine.rs       SimEngine struct: new(), step(), reset()
│   ├── agent.rs            Agent (#[repr(C)]), AgentType, Action, Intention enums
│   ├── grid.rs             Grid<f32> resource environment
│   ├── agent_buffers.rs    SoA Vec<f32> buffer writer
│   └── event_log.rs        SimEvent, EventLog
├── decision_rules/
│   ├── mod.rs              DecisionRule trait (object-safe) + make_rule_set()
│   ├── reactive.rs         IF-THEN reactive rules
│   ├── bounded.rs          Bounded rationality (satisficing)
│   └── bdi.rs              Beliefs/Desires/Intentions architecture
├── spatial/spatial_grid.rs SpatialGrid: O(1) bucket-based neighbor lookup
├── rng/mulberry32.rs       Seeded Mulberry32 PRNG
├── statistics/stats.rs     SimStats, gini(), mean_energy(), action_dist()
├── stochastic/
│   ├── bayesian.rs         Bayesian inference
│   ├── ctmc.rs             Continuous-time Markov chains
│   ├── dtmc.rs             Discrete-time Markov chains
│   ├── distributions.rs    Probability distributions
│   ├── monte_carlo.rs      Monte Carlo methods
│   ├── poisson.rs          Poisson process
│   ├── sde.rs              Stochastic differential equations
│   └── survival.rs         Survival analysis
└── tests/                  Integration tests (engine, bayesian, ctmc, dtmc, poisson, stochastic, survival)
```

### Key Rust Patterns

- All hot-path structs: `#[repr(C)]` for predictable layout
- Agent coordinates: `f32` not `f64` (2× SIMD throughput on wasm32)
- Hot-path data: `Vec<f32>` / `Vec<u8>` — never cross JS/WASM boundary with agent structs in the tick loop
- `serde` / JSON: acceptable only for params (cold path); banned inside `SimEngine::step()`
- WASM API functions: never use `.unwrap()` — panics are unrecoverable in WASM
- `unsafe`: only in `wasm_api.rs` for typed array views; all blocks must be commented
- Feature-gate browser-specific code: `#[cfg(target_arch = "wasm32")]`
- Panic handler: `console_error_panic_hook::set_once()` in `WasmSimEngine::new()`

### JS/WASM Bridge Rules

- All WASM calls go through `src/bridge/simBridge.js` — no other module imports WASM
- Hot-path agent data: `Float32Array` view over SAB — never copy, never serialize
- Control messages: `Worker.postMessage` with structured clone (params, commands)
- WASM errors: catch in `simBridge.js`, dispatch `'engine:error'` via `ABMDispatch`
- Never block the main thread waiting for WASM — all calls async via Worker

### D3 / JS Coding Standards

- D3 imports: always from submodule (`'d3-selection'`, not `'d3'`)
- DOM building: `d3.select`/`d3.create` only — never `innerHTML`, never `createElement`
- Events: `ABMDispatch` (d3-dispatch) only — never `addEventListener` directly
- Sim loop: `d3.timer` in Worker — never `setInterval`, never bare `rAF` on main thread
- Transitions: `duration(isRunning ? 0 : 250)` — zero during active simulation
- Logging: `src/utils/logger.js` only — `console.log` is banned

## Cross-Origin Isolation (Required for SharedArrayBuffer)

`vite.config.js` and `netlify.toml` must serve these headers:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```
Without these, `SharedArrayBuffer` is unavailable and the sim worker cannot start.

## Commit Format

```
type(scope): description

Types:  feat | fix | perf | test | refactor | docs | chore
Scopes: engine | stochastic | bridge | renderer | charts | ui | worker | e2e | config
```

## Banned (Never Do)

- Any JS framework: React, Vue, Angular, Svelte, Preact
- `std::thread` in WASM builds (wasm32 is single-threaded within the Worker)
- `.unwrap()` in WASM API functions
- `serde` JSON serialization inside `SimEngine::step()` hot path
- Direct WASM imports outside `src/bridge/simBridge.js`
- `console.log` (use `logger.js`)
- `innerHTML` (use `d3.create`)
- `f64` for agent position coordinates (use `f32`)
