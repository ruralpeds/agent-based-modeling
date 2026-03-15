use wasm_bindgen::prelude::*;
use js_sys::Float32Array;
use crate::engine::sim_engine::{SimEngine, SimParams};
use crate::hospital::{HospitalSimEngine, HospitalParams};

#[wasm_bindgen]
pub struct WasmSimEngine {
    inner: SimEngine,
}

#[wasm_bindgen]
impl WasmSimEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(params_json: &str, seed: u32) -> Result<WasmSimEngine, JsValue> {
        console_error_panic_hook::set_once();
        let params: SimParams = serde_json::from_str(params_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmSimEngine { inner: SimEngine::new(params, seed) })
    }

    /// Advance one tick. Returns agent count.
    pub fn step(&mut self) -> u32 {
        self.inner.step();
        self.inner.agent_count() as u32
    }

    /// Returns Float32Array VIEW into internal SoA buffer.
    /// Read before next step() — data is overwritten.
    pub fn get_agent_buffer(&mut self) -> Float32Array {
        let buf = self.inner.write_agent_buffer();
        // SAFETY: buffer lives inside WasmSimEngine for the duration of the view
        unsafe { Float32Array::view(buf) }
    }

    pub fn set_params(&mut self, params_json: &str) -> Result<(), JsValue> {
        let params: SimParams = serde_json::from_str(params_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.set_params(params);
        Ok(())
    }

    pub fn reset(&mut self, seed: u32) {
        self.inner.reset(seed);
    }

    pub fn get_tick(&self) -> u32 { self.inner.tick() as u32 }
    pub fn get_prey_count(&self) -> u32 { self.inner.prey_count() as u32 }
    pub fn get_pred_count(&self) -> u32 { self.inner.pred_count() as u32 }
    pub fn get_agent_count(&self) -> u32 { self.inner.agent_count() as u32 }

    pub fn get_stats_json(&self) -> String {
        serde_json::to_string(&self.inner.stats()).unwrap_or_default()
    }
}

// ─── WasmHospitalEngine ───────────────────────────────────────────────────────

/// WASM binding for the healthcare HAI simulation engine.
/// All methods are async-safe: call from a Web Worker, never from main thread.
#[wasm_bindgen]
pub struct WasmHospitalEngine {
    inner: HospitalSimEngine,
}

#[wasm_bindgen]
impl WasmHospitalEngine {
    /// Construct a new hospital engine from a JSON-serialised `HospitalParams`.
    /// Pass `"{}"` to use all defaults.
    #[wasm_bindgen(constructor)]
    pub fn new(params_json: &str, seed: u32) -> Result<WasmHospitalEngine, JsValue> {
        console_error_panic_hook::set_once();
        let params: HospitalParams = serde_json::from_str(params_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmHospitalEngine { inner: HospitalSimEngine::new(params, seed) })
    }

    /// Advance one simulation tick (≈ 5 min). Returns current patient census.
    pub fn step(&mut self) -> u32 {
        self.inner.step();
        self.inner.patient_count() as u32
    }

    /// Returns a Float32Array VIEW over the SoA patient buffer.
    /// Layout: 12 × N floats (see `patient_field` constants).
    /// **Must be read before calling `step()` again** — buffer is overwritten.
    pub fn get_patient_buffer(&mut self) -> Float32Array {
        let buf = self.inner.write_patient_buffer();
        // SAFETY: `buf` is a slice into the Vec inside `WasmHospitalEngine`.
        // The view is valid until the next call to `step()` or `get_patient_buffer()`.
        unsafe { Float32Array::view(buf) }
    }

    /// Update engine parameters at runtime (e.g. adjust DTMC rates).
    pub fn set_params(&mut self, params_json: &str) -> Result<(), JsValue> {
        let params: HospitalParams = serde_json::from_str(params_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner.set_params(params);
        Ok(())
    }

    pub fn reset(&mut self, seed: u32) {
        self.inner.reset(seed);
    }

    /// JSON-serialised `HospitalStats` snapshot for the most recent tick.
    pub fn get_stats_json(&self) -> String {
        serde_json::to_string(&self.inner.stats()).unwrap_or_default()
    }

    pub fn get_tick(&self)           -> u32 { self.inner.tick()           as u32 }
    pub fn get_patient_count(&self)  -> u32 { self.inner.patient_count()  as u32 }
    pub fn get_cumulative_hai(&self) -> u32 { self.inner.cumulative_hai()         }
}
