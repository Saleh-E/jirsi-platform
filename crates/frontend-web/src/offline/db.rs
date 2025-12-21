
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen::to_value;


#[wasm_bindgen(module = "/assets/offline_adapter.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn initOfflineDb() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    fn executeSql(sql: &str, params: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug)]
pub struct LocalDatabase {
    // We don't hold the JS object directly here, it's global in JS for now
    // In a cleaner implementation we'd pass a handle
    initialized: bool,
}

impl LocalDatabase {
    pub async fn new() -> Result<Self, String> {
        match initOfflineDb().await {
            Ok(val) => {
                if val.is_truthy() {
                     Ok(Self { initialized: true })
                } else {
                    Err("Failed to init OPFS DB".to_string())
                }
            },
            Err(e) => Err(format!("JS Error: {:?}", e))
        }
    }

    pub fn execute(&self, sql: &str) -> Result<(), String> {
        executeSql(sql, JsValue::NULL)
            .map(|_| ())
            .map_err(|e| format!("{:?}", e))
    }

    pub fn execute_with_params(&self, sql: &str, params: serde_json::Value) -> Result<(), String> {
        let js_params = to_value(&params).map_err(|e| e.to_string())?;
        executeSql(sql, js_params)
            .map(|_| ())
            .map_err(|e| format!("{:?}", e))
    }
    
    pub fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>, String> {
        let res = executeSql(sql, JsValue::NULL)
            .map_err(|e| format!("{:?}", e))?;
            
        serde_wasm_bindgen::from_value(res)
            .map_err(|e| e.to_string())
    }
}
