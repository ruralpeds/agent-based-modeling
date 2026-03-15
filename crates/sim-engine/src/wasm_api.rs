use wasm_bindgen::prelude::*;
use js_sys::Float32Array;
use crate::engine::sim_engine::{SimEngine, SimParams};

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
