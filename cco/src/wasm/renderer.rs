//! Canvas rendering for terminal output

use web_sys::CanvasRenderingContext2d;
use wasm_bindgen::JsValue;
use crate::terminal::{TerminalBuffer, Color};

/// Canvas renderer for terminal buffer
pub struct CanvasRenderer {
    // Font configuration
    font_family: String,
    font_size: f64,
    char_width: f64,
    char_height: f64,
    line_height: f64,
}

impl CanvasRenderer {
    /// Create new renderer
    pub fn new() -> Self {
        // Default monospace font settings
        let font_size = 14.0;
        let char_width = font_size * 0.6;  // Approximate for monospace
        let char_height = font_size;
        let line_height = font_size * 1.2;

        CanvasRenderer {
            font_family: "Fira Code, Consolas, Monaco, monospace".to_string(),
            font_size,
            char_width,
            char_height,
            line_height,
        }
    }

    /// Render terminal buffer to canvas
    pub fn render(
        &self,
        buffer: &TerminalBuffer,
        context: &CanvasRenderingContext2d,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Result<(), JsValue> {
        // Clear canvas
        context.clear_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);

        // Set default background
        context.set_fill_style(&JsValue::from_str("#000000"));
        context.fill_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);

        // Set font
        let font_str = format!("{}px {}", self.font_size, self.font_family);
        context.set_font(&font_str);

        // Set text baseline for consistent rendering
        context.set_text_baseline("top");

        let (cols, rows) = buffer.dimensions();

        // Render each cell
        for row in 0..rows {
            for col in 0..cols {
                if let Some(cell) = buffer.get_cell(col, row) {
                    let x = col as f64 * self.char_width;
                    let y = row as f64 * self.line_height;

                    // Skip empty cells with default background
                    if cell.char == ' ' && cell.bg == Color::BLACK {
                        continue;
                    }

                    // Draw background if not black
                    if cell.bg != Color::BLACK {
                        context.set_fill_style(&color_to_css(&cell.bg));
                        context.fill_rect(x, y, self.char_width, self.line_height);
                    }

                    // Draw character if not space
                    if cell.char != ' ' {
                        // Apply text attributes
                        let mut font_style = String::new();
                        if cell.attrs.italic {
                            font_style.push_str("italic ");
                        }
                        if cell.attrs.bold {
                            font_style.push_str("bold ");
                        }
                        let font_str = format!("{} {}px {}", font_style, self.font_size, self.font_family);
                        context.set_font(&font_str);

                        // Set text color
                        let fg_color = if cell.attrs.reverse {
                            &cell.bg
                        } else {
                            &cell.fg
                        };
                        context.set_fill_style(&color_to_css(fg_color));

                        // Draw the character
                        if !cell.attrs.hidden {
                            context.fill_text(&cell.char.to_string(), x, y)?;
                        }

                        // Draw underline
                        if cell.attrs.underline {
                            context.set_stroke_style(&color_to_css(fg_color));
                            context.set_line_width(1.0);
                            context.begin_path();
                            context.move_to(x, y + self.line_height - 2.0);
                            context.line_to(x + self.char_width, y + self.line_height - 2.0);
                            context.stroke();
                        }

                        // Draw strikethrough
                        if cell.attrs.strikethrough {
                            context.set_stroke_style(&color_to_css(fg_color));
                            context.set_line_width(1.0);
                            context.begin_path();
                            context.move_to(x, y + self.line_height / 2.0);
                            context.line_to(x + self.char_width, y + self.line_height / 2.0);
                            context.stroke();
                        }
                    }
                }
            }
        }

        // Draw cursor
        let (cursor_x, cursor_y) = buffer.get_cursor();
        let x = cursor_x as f64 * self.char_width;
        let y = cursor_y as f64 * self.line_height;

        // Blinking block cursor
        context.set_fill_style(&JsValue::from_str("#00FF00"));
        context.set_global_alpha(0.5);
        context.fill_rect(x, y, self.char_width, self.line_height);
        context.set_global_alpha(1.0);

        Ok(())
    }

    /// Update font metrics
    pub fn set_font_size(&mut self, size: f64) {
        self.font_size = size;
        self.char_width = size * 0.6;
        self.char_height = size;
        self.line_height = size * 1.2;
    }

    /// Get character dimensions
    pub fn get_char_dimensions(&self) -> (f64, f64) {
        (self.char_width, self.line_height)
    }
}

/// Convert Color to CSS color string
fn color_to_css(color: &Color) -> JsValue {
    let css = format!("rgb({}, {}, {})", color.r, color.g, color.b);
    JsValue::from_str(&css)
}