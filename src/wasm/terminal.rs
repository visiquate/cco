//! Terminal buffer management
//!
//! Cell-based terminal buffer with scrollback support

use std::collections::VecDeque;

/// Terminal color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };
}

/// Cell attributes
#[derive(Debug, Clone, Copy, Default)]
pub struct CellAttrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
}

/// Individual terminal cell
#[derive(Debug, Clone)]
pub struct Cell {
    pub char: char,
    pub fg: Color,
    pub bg: Color,
    pub attrs: CellAttrs,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            char: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            attrs: CellAttrs::default(),
        }
    }
}

/// Terminal buffer state
pub struct TerminalBuffer {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
    cursor_x: u16,
    cursor_y: u16,
    scrollback: VecDeque<Vec<Cell>>,
    max_scrollback: usize,
    // Current text attributes for new characters
    current_fg: Color,
    current_bg: Color,
    current_attrs: CellAttrs,
}

impl TerminalBuffer {
    /// Create new terminal buffer
    pub fn new(width: u16, height: u16) -> Self {
        let capacity = (width * height) as usize;
        let mut cells = Vec::with_capacity(capacity);
        cells.resize(capacity, Cell::default());

        TerminalBuffer {
            cells,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            scrollback: VecDeque::new(),
            max_scrollback: 10000,
            current_fg: Color::WHITE,
            current_bg: Color::BLACK,
            current_attrs: CellAttrs::default(),
        }
    }

    /// Get cell at position
    pub fn get_cell(&self, x: u16, y: u16) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.cells.get(index)
    }

    /// Get mutable cell at position
    pub fn get_cell_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y * self.width + x) as usize;
        self.cells.get_mut(index)
    }

    /// Write character at current cursor position
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => self.new_line(),
            '\r' => self.cursor_x = 0,
            '\t' => {
                // Move to next tab stop (every 8 columns)
                let next_tab = ((self.cursor_x / 8) + 1) * 8;
                self.cursor_x = next_tab.min(self.width - 1);
            }
            '\x08' => {
                // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            _ => {
                if let Some(cell) = self.get_cell_mut(self.cursor_x, self.cursor_y) {
                    cell.char = ch;
                    cell.fg = self.current_fg;
                    cell.bg = self.current_bg;
                    cell.attrs = self.current_attrs;
                }

                self.cursor_x += 1;
                if self.cursor_x >= self.width {
                    self.cursor_x = 0;
                    self.new_line();
                }
            }
        }
    }

    /// Write string to buffer
    pub fn write_str(&mut self, text: &str) {
        for ch in text.chars() {
            self.write_char(ch);
        }
    }

    /// Move to new line
    fn new_line(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y >= self.height {
            self.scroll_up();
            self.cursor_y = self.height - 1;
        }
    }

    /// Scroll buffer up by one line
    fn scroll_up(&mut self) {
        // Save first line to scrollback
        let mut line = Vec::with_capacity(self.width as usize);
        for x in 0..self.width {
            if let Some(cell) = self.get_cell(x, 0) {
                line.push(cell.clone());
            }
        }

        self.scrollback.push_back(line);
        if self.scrollback.len() > self.max_scrollback {
            self.scrollback.pop_front();
        }

        // Shift all lines up
        for y in 0..(self.height - 1) {
            for x in 0..self.width {
                let src_index = ((y + 1) * self.width + x) as usize;
                let dst_index = (y * self.width + x) as usize;
                self.cells[dst_index] = self.cells[src_index].clone();
            }
        }

        // Clear bottom line
        for x in 0..self.width {
            let index = ((self.height - 1) * self.width + x) as usize;
            self.cells[index] = Cell::default();
        }
    }

    /// Clear entire buffer
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Clear from cursor to end of line
    pub fn clear_to_eol(&mut self) {
        for x in self.cursor_x..self.width {
            if let Some(cell) = self.get_cell_mut(x, self.cursor_y) {
                *cell = Cell::default();
            }
        }
    }

    /// Clear from cursor to end of screen
    pub fn clear_to_eos(&mut self) {
        // Clear rest of current line
        self.clear_to_eol();

        // Clear all lines below
        for y in (self.cursor_y + 1)..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.get_cell_mut(x, y) {
                    *cell = Cell::default();
                }
            }
        }
    }

    /// Resize buffer
    pub fn resize(&mut self, new_width: u16, new_height: u16) {
        let new_capacity = (new_width * new_height) as usize;
        let mut new_cells = Vec::with_capacity(new_capacity);
        new_cells.resize(new_capacity, Cell::default());

        // Copy existing cells to new buffer
        let copy_width = self.width.min(new_width);
        let copy_height = self.height.min(new_height);

        for y in 0..copy_height {
            for x in 0..copy_width {
                let old_index = (y * self.width + x) as usize;
                let new_index = (y * new_width + x) as usize;
                if old_index < self.cells.len() && new_index < new_cells.len() {
                    new_cells[new_index] = self.cells[old_index].clone();
                }
            }
        }

        self.cells = new_cells;
        self.width = new_width;
        self.height = new_height;

        // Adjust cursor position
        self.cursor_x = self.cursor_x.min(new_width - 1);
        self.cursor_y = self.cursor_y.min(new_height - 1);
    }

    /// Set cursor position
    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.cursor_x = x.min(self.width - 1);
        self.cursor_y = y.min(self.height - 1);
    }

    /// Get cursor position
    pub fn get_cursor(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }

    /// Set foreground color
    pub fn set_fg_color(&mut self, color: Color) {
        self.current_fg = color;
    }

    /// Set background color
    pub fn set_bg_color(&mut self, color: Color) {
        self.current_bg = color;
    }

    /// Set text attributes
    pub fn set_attrs(&mut self, attrs: CellAttrs) {
        self.current_attrs = attrs;
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}