//! Event handling for the TUI dashboard

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Events that can occur in the TUI
#[derive(Debug, Clone)]
pub enum Event {
    /// Tick event (for refresh updates)
    Tick,
    /// Key event
    Key(KeyEvent),
    /// Window resize event
    Resize(u16, u16),
    /// Other events
    Other,
}

/// Handles terminal events
pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    _tx: mpsc::Sender<Event>,
    stop_flag: Arc<AtomicBool>,
}

impl EventHandler {
    /// Create a new event handler with specified poll interval (ms)
    pub fn new(poll_interval_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = Arc::clone(&stop_flag);

        // Spawn event handling thread
        thread::spawn(move || {
            let poll_duration = Duration::from_millis(poll_interval_ms);

            loop {
                // Check if we should stop
                if stop_flag_clone.load(Ordering::Relaxed) {
                    break;
                }

                // Poll for key events
                if event::poll(poll_duration).ok().unwrap_or(false) {
                    if let Ok(event::Event::Key(key)) = event::read() {
                        let _ = tx_clone.send(Event::Key(key));
                    } else if let Ok(event::Event::Resize(width, height)) = event::read() {
                        let _ = tx_clone.send(Event::Resize(width, height));
                    }
                }

                // Send tick events for refresh
                let _ = tx_clone.send(Event::Tick);
            }
        });

        Self {
            rx,
            _tx: tx,
            stop_flag,
        }
    }

    /// Get next event (non-blocking)
    pub fn next(&self) -> Result<Option<Event>> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Ok(None),
        }
    }

    /// Stop the event handler
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Check if a key event is the quit command
pub fn is_quit_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('c') | KeyCode::Char('C')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            true
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => true,
        KeyCode::Esc => true,
        _ => false,
    }
}

/// Check if a key event is a navigation key
pub fn is_next_tab_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Right | KeyCode::Tab => true,
        KeyCode::Char('l') | KeyCode::Char('L') => true,
        _ => false,
    }
}

/// Check if a key event is a previous navigation key
pub fn is_prev_tab_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left => true,
        KeyCode::BackTab => true,
        KeyCode::Char('h') | KeyCode::Char('H') => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let _handler = EventHandler::new(250);
        // Handler is created successfully
    }

    #[test]
    fn test_is_quit_key() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_quit_key(key));

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(is_quit_key(key));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(is_quit_key(key));

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!is_quit_key(key));
    }

    #[test]
    fn test_is_next_tab_key() {
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(is_next_tab_key(key));

        let key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert!(is_next_tab_key(key));

        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        assert!(is_next_tab_key(key));

        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert!(!is_next_tab_key(key));
    }

    #[test]
    fn test_is_prev_tab_key() {
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        assert!(is_prev_tab_key(key));

        let key = KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE);
        assert!(is_prev_tab_key(key));

        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        assert!(is_prev_tab_key(key));

        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert!(!is_prev_tab_key(key));
    }
}
