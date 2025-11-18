//! WASM Terminal Implementation
//!
//! High-performance terminal emulator compiled to WebAssembly.
//! Provides VT100/ANSI parsing via VTE crate with Canvas rendering.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use std::cell::RefCell;
use std::rc::Rc;

mod terminal;
mod parser;
mod renderer;
mod bindings;

use terminal::{TerminalBuffer, Cell};
use parser::AnsiParser;
use renderer::CanvasRenderer;

/// Main WASM Terminal structure exposed to JavaScript
#[wasm_bindgen]
pub struct WasmTerminal {
    buffer: Rc<RefCell<TerminalBuffer>>,
    parser: Rc<RefCell<AnsiParser>>,
    renderer: CanvasRenderer,
    cols: u16,
    rows: u16,
}

#[wasm_bindgen]
impl WasmTerminal {
    /// Create a new WASM terminal instance
    #[wasm_bindgen(constructor)]
    pub fn new(cols: u16, rows: u16) -> Result<WasmTerminal, JsValue> {
        // Set panic hook for better error messages in browser console
        console_error_panic_hook::set_once();

        web_sys::console::log_1(&format!("WasmTerminal::new({}, {})", cols, rows).into());

        let buffer = Rc::new(RefCell::new(TerminalBuffer::new(cols, rows)));
        let parser = Rc::new(RefCell::new(AnsiParser::new(Rc::clone(&buffer))));
        let renderer = CanvasRenderer::new();

        Ok(WasmTerminal {
            buffer,
            parser,
            renderer,
            cols,
            rows,
        })
    }

    /// Parse input data and update terminal buffer
    pub fn parse(&mut self, input: &str) {
        web_sys::console::log_1(&format!("WasmTerminal::parse(len={})", input.len()).into());
        self.parser.borrow_mut().parse(input.as_bytes());
    }

    /// Render terminal buffer to canvas element
    pub fn render_to_canvas(&self, canvas_id: &str) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("WasmTerminal::render_to_canvas({})", canvas_id).into());

        // Get canvas element
        let document = web_sys::window()
            .ok_or("No window object")?
            .document()
            .ok_or("No document object")?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or(format!("Canvas {} not found", canvas_id))?
            .dyn_into::<HtmlCanvasElement>()?;

        let context = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Render buffer to canvas
        self.renderer.render(&self.buffer.borrow(), &context, canvas.width(), canvas.height())?;

        Ok(())
    }

    /// Get current terminal output as string (for testing)
    pub fn get_output(&self) -> String {
        let buffer = self.buffer.borrow();
        let mut output = String::new();

        for row in 0..self.rows {
            for col in 0..self.cols {
                if let Some(cell) = buffer.get_cell(col, row) {
                    output.push(cell.char);
                }
            }
            output.push('\n');
        }

        output.trim_end().to_string()
    }

    /// Clear the terminal buffer
    pub fn clear(&mut self) {
        self.buffer.borrow_mut().clear();
    }

    /// Resize terminal
    pub fn resize(&mut self, cols: u16, rows: u16) {
        web_sys::console::log_1(&format!("WasmTerminal::resize({}, {})", cols, rows).into());
        self.cols = cols;
        self.rows = rows;
        self.buffer.borrow_mut().resize(cols, rows);
    }

    /// Write text at current cursor position
    pub fn write(&mut self, text: &str) {
        self.buffer.borrow_mut().write_str(text);
    }

    /// Get terminal dimensions
    pub fn get_cols(&self) -> u16 {
        self.cols
    }

    pub fn get_rows(&self) -> u16 {
        self.rows
    }
}

/// Enable panic hook for better debugging
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}