//! VT100/ANSI escape sequence parser
//!
//! Wraps the VTE crate for high-performance terminal parsing

use std::rc::Rc;
use std::cell::RefCell;
use vte::{Parser, Perform};
use crate::terminal::{TerminalBuffer, Color, CellAttrs};

/// ANSI parser using VTE crate
pub struct AnsiParser {
    parser: Parser,
    buffer: Rc<RefCell<TerminalBuffer>>,
}

impl AnsiParser {
    /// Create new parser
    pub fn new(buffer: Rc<RefCell<TerminalBuffer>>) -> Self {
        AnsiParser {
            parser: Parser::new(),
            buffer,
        }
    }

    /// Parse input bytes
    pub fn parse(&mut self, input: &[u8]) {
        let mut performer = AnsiPerformer {
            buffer: Rc::clone(&self.buffer),
            // Track intermediate state for escape sequences
            intermediates: Vec::new(),
            params: Vec::new(),
        };

        for byte in input {
            self.parser.advance(&mut performer, *byte);
        }
    }
}

/// VTE Perform trait implementation
struct AnsiPerformer {
    buffer: Rc<RefCell<TerminalBuffer>>,
    intermediates: Vec<u8>,
    params: Vec<i64>,
}

impl Perform for AnsiPerformer {
    fn print(&mut self, c: char) {
        self.buffer.borrow_mut().write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                // Backspace
                let mut buffer = self.buffer.borrow_mut();
                let (x, y) = buffer.get_cursor();
                if x > 0 {
                    buffer.set_cursor(x - 1, y);
                }
            }
            0x09 => {
                // Tab
                self.buffer.borrow_mut().write_char('\t');
            }
            0x0A | 0x0B | 0x0C => {
                // Line feed, vertical tab, form feed
                self.buffer.borrow_mut().write_char('\n');
            }
            0x0D => {
                // Carriage return
                self.buffer.borrow_mut().write_char('\r');
            }
            0x07 => {
                // Bell (BEL) - just log for now
                web_sys::console::log_1(&"Terminal bell".into());
            }
            _ => {
                // Other control characters - ignore for now
            }
        }
    }

    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool) {
        // OSC and other sequences - not implemented yet
    }

    fn put(&mut self, _byte: u8) {
        // DCS put - not implemented
    }

    fn unhook(&mut self) {
        // DCS unhook - not implemented
    }

    fn osc_dispatch(&mut self, params: &[&[u8]]) {
        // Handle Operating System Commands (OSC)
        if params.is_empty() {
            return;
        }

        match params[0] {
            b"0" | b"2" => {
                // Set window title
                if params.len() > 1 {
                    if let Ok(title) = std::str::from_utf8(params[1]) {
                        web_sys::console::log_1(&format!("Set window title: {}", title).into());
                    }
                }
            }
            _ => {
                // Other OSC sequences not implemented
            }
        }
    }

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], _ignore: bool, action: char) {
        let mut buffer = self.buffer.borrow_mut();

        match action {
            // Cursor movement
            'A' => {
                // Cursor up
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                let (x, y) = buffer.get_cursor();
                buffer.set_cursor(x, y.saturating_sub(n));
            }
            'B' => {
                // Cursor down
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                let (x, y) = buffer.get_cursor();
                let (_, height) = buffer.dimensions();
                buffer.set_cursor(x, (y + n).min(height - 1));
            }
            'C' => {
                // Cursor forward
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                let (x, y) = buffer.get_cursor();
                let (width, _) = buffer.dimensions();
                buffer.set_cursor((x + n).min(width - 1), y);
            }
            'D' => {
                // Cursor backward
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                let (x, y) = buffer.get_cursor();
                buffer.set_cursor(x.saturating_sub(n), y);
            }
            'H' | 'f' => {
                // Cursor position
                let y = params.get(0).copied().unwrap_or(1).max(1) as u16 - 1;
                let x = params.get(1).copied().unwrap_or(1).max(1) as u16 - 1;
                buffer.set_cursor(x, y);
            }
            'J' => {
                // Erase display
                match params.get(0).copied().unwrap_or(0) {
                    0 => buffer.clear_to_eos(),
                    1 => {
                        // Clear from cursor to beginning of screen
                        // Not implemented yet - would need clear_to_bos
                    }
                    2 | 3 => buffer.clear(),
                    _ => {}
                }
            }
            'K' => {
                // Erase line
                match params.get(0).copied().unwrap_or(0) {
                    0 => buffer.clear_to_eol(),
                    1 => {
                        // Clear from cursor to beginning of line
                        // Not implemented yet - would need clear_to_bol
                    }
                    2 => {
                        // Clear entire line
                        let (_, y) = buffer.get_cursor();
                        buffer.set_cursor(0, y);
                        buffer.clear_to_eol();
                    }
                    _ => {}
                }
            }
            'm' => {
                // Select Graphic Rendition (SGR) - colors and attributes
                if params.is_empty() {
                    // Reset all attributes
                    buffer.set_fg_color(Color::WHITE);
                    buffer.set_bg_color(Color::BLACK);
                    buffer.set_attrs(CellAttrs::default());
                } else {
                    for &param in params {
                        handle_sgr_param(&mut buffer, param);
                    }
                }
            }
            's' => {
                // Save cursor position (ANSI.SYS)
                // Would need to store cursor position
            }
            'u' => {
                // Restore cursor position (ANSI.SYS)
                // Would need to restore saved position
            }
            _ => {
                // Unhandled CSI sequence
                web_sys::console::log_1(&format!("Unhandled CSI: {:?} {:?} {}", params, intermediates, action).into());
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        let mut buffer = self.buffer.borrow_mut();

        match byte {
            b'M' => {
                // Reverse index - move cursor up, scroll if at top
                let (x, y) = buffer.get_cursor();
                if y > 0 {
                    buffer.set_cursor(x, y - 1);
                } else {
                    // Would need to implement scroll_down
                }
            }
            b'E' => {
                // Next line
                buffer.write_char('\n');
            }
            b'D' => {
                // Index - move cursor down, scroll if at bottom
                buffer.write_char('\n');
            }
            b'c' => {
                // Reset to initial state
                buffer.clear();
                buffer.set_cursor(0, 0);
            }
            _ => {
                // Unhandled escape sequence
                web_sys::console::log_1(&format!("Unhandled ESC: {:?} {}", intermediates, byte).into());
            }
        }
    }
}

/// Handle SGR (Select Graphic Rendition) parameters
fn handle_sgr_param(buffer: &mut TerminalBuffer, param: i64) {
    match param {
        0 => {
            // Reset all
            buffer.set_fg_color(Color::WHITE);
            buffer.set_bg_color(Color::BLACK);
            buffer.set_attrs(CellAttrs::default());
        }
        1 => {
            // Bold
            let mut attrs = CellAttrs::default();
            attrs.bold = true;
            buffer.set_attrs(attrs);
        }
        3 => {
            // Italic
            let mut attrs = CellAttrs::default();
            attrs.italic = true;
            buffer.set_attrs(attrs);
        }
        4 => {
            // Underline
            let mut attrs = CellAttrs::default();
            attrs.underline = true;
            buffer.set_attrs(attrs);
        }
        5 => {
            // Blink
            let mut attrs = CellAttrs::default();
            attrs.blink = true;
            buffer.set_attrs(attrs);
        }
        7 => {
            // Reverse
            let mut attrs = CellAttrs::default();
            attrs.reverse = true;
            buffer.set_attrs(attrs);
        }
        8 => {
            // Hidden
            let mut attrs = CellAttrs::default();
            attrs.hidden = true;
            buffer.set_attrs(attrs);
        }
        9 => {
            // Strikethrough
            let mut attrs = CellAttrs::default();
            attrs.strikethrough = true;
            buffer.set_attrs(attrs);
        }
        // Foreground colors
        30 => buffer.set_fg_color(Color::BLACK),
        31 => buffer.set_fg_color(Color::RED),
        32 => buffer.set_fg_color(Color::GREEN),
        33 => buffer.set_fg_color(Color::YELLOW),
        34 => buffer.set_fg_color(Color::BLUE),
        35 => buffer.set_fg_color(Color::MAGENTA),
        36 => buffer.set_fg_color(Color::CYAN),
        37 => buffer.set_fg_color(Color::WHITE),
        // Background colors
        40 => buffer.set_bg_color(Color::BLACK),
        41 => buffer.set_bg_color(Color::RED),
        42 => buffer.set_bg_color(Color::GREEN),
        43 => buffer.set_bg_color(Color::YELLOW),
        44 => buffer.set_bg_color(Color::BLUE),
        45 => buffer.set_bg_color(Color::MAGENTA),
        46 => buffer.set_bg_color(Color::CYAN),
        47 => buffer.set_bg_color(Color::WHITE),
        // Bright foreground colors
        90..=97 => {
            // Map to regular colors for now (could make brighter)
            let color = match param {
                90 => Color::BLACK,
                91 => Color::RED,
                92 => Color::GREEN,
                93 => Color::YELLOW,
                94 => Color::BLUE,
                95 => Color::MAGENTA,
                96 => Color::CYAN,
                97 => Color::WHITE,
                _ => Color::WHITE,
            };
            buffer.set_fg_color(color);
        }
        // Bright background colors
        100..=107 => {
            let color = match param {
                100 => Color::BLACK,
                101 => Color::RED,
                102 => Color::GREEN,
                103 => Color::YELLOW,
                104 => Color::BLUE,
                105 => Color::MAGENTA,
                106 => Color::CYAN,
                107 => Color::WHITE,
                _ => Color::BLACK,
            };
            buffer.set_bg_color(color);
        }
        _ => {
            // Other SGR params not implemented
            web_sys::console::log_1(&format!("Unhandled SGR param: {}", param).into());
        }
    }
}