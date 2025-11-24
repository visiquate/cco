//! JavaScript FFI bindings for WASM terminal

use wasm_bindgen::prelude::*;

/// JavaScript callbacks for terminal events
#[wasm_bindgen]
extern "C" {
    /// Console logging
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    /// Error logging
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);

    /// Performance timing
    #[wasm_bindgen(js_namespace = performance)]
    pub fn now() -> f64;
}

/// Macro for console logging
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into())
    }
}

/// Macro for error logging
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => {
        web_sys::console::error_1(&format!($($t)*).into())
    }
}

/// Terminal event types
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Input,
    Output,
    Resize,
    Focus,
    Blur,
}

/// Terminal configuration from JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub cols: u16,
    pub rows: u16,
    pub font_size: f64,
    pub cursor_blink: bool,
    pub scrollback_lines: u32,
}

#[wasm_bindgen]
impl TerminalConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TerminalConfig {
        TerminalConfig {
            cols: 80,
            rows: 24,
            font_size: 14.0,
            cursor_blink: true,
            scrollback_lines: 10000,
        }
    }

    pub fn set_dimensions(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
    }

    pub fn set_font_size(&mut self, size: f64) {
        self.font_size = size;
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript interface for terminal callbacks
#[wasm_bindgen]
pub struct TerminalCallbacks {
    on_data: Option<js_sys::Function>,
    on_resize: Option<js_sys::Function>,
    on_title: Option<js_sys::Function>,
}

#[wasm_bindgen]
impl TerminalCallbacks {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TerminalCallbacks {
        TerminalCallbacks {
            on_data: None,
            on_resize: None,
            on_title: None,
        }
    }

    pub fn set_on_data(&mut self, callback: js_sys::Function) {
        self.on_data = Some(callback);
    }

    pub fn set_on_resize(&mut self, callback: js_sys::Function) {
        self.on_resize = Some(callback);
    }

    pub fn set_on_title(&mut self, callback: js_sys::Function) {
        self.on_title = Some(callback);
    }

    pub fn trigger_data(&self, data: &str) {
        if let Some(ref callback) = self.on_data {
            let this = JsValue::null();
            let arg = JsValue::from_str(data);
            let _ = callback.call1(&this, &arg);
        }
    }

    pub fn trigger_resize(&self, cols: u16, rows: u16) {
        if let Some(ref callback) = self.on_resize {
            let this = JsValue::null();
            let cols_val = JsValue::from_f64(cols as f64);
            let rows_val = JsValue::from_f64(rows as f64);
            let _ = callback.call2(&this, &cols_val, &rows_val);
        }
    }

    pub fn trigger_title(&self, title: &str) {
        if let Some(ref callback) = self.on_title {
            let this = JsValue::null();
            let arg = JsValue::from_str(title);
            let _ = callback.call1(&this, &arg);
        }
    }
}

/// Utility to measure performance
#[wasm_bindgen]
pub struct PerformanceTimer {
    start_time: f64,
}

#[wasm_bindgen]
impl PerformanceTimer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PerformanceTimer {
        PerformanceTimer {
            start_time: performance::now(),
        }
    }

    pub fn elapsed(&self) -> f64 {
        performance::now() - self.start_time
    }

    pub fn reset(&mut self) {
        self.start_time = performance::now();
    }
}