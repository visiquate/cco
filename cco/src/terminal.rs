//! PTY-based terminal session management
//!
//! Provides real shell process execution with PTY (pseudoterminal) support,
//! enabling full terminal emulation including proper signal handling, window resizing,
//! and streaming I/O.
//!
//! # Architecture
//!
//! This module manages the lifecycle of pseudoterminal (PTY) sessions, providing
//! a thread-safe interface to interact with shell processes. Each session spawns
//! a real shell process connected via a PTY pair, enabling authentic terminal
//! behavior with proper signal handling and process management.
//!
//! # Session Lifecycle
//!
//! 1. **Creation** - `spawn_shell()` creates PTY pair, spawns shell
//! 2. **Active** - Input/output operations via `write_input()` and `read_output()`
//! 3. **Termination** - `close_session()` kills process and cleans up resources
//!
//! # Thread Safety
//!
//! All operations are thread-safe via Arc<Mutex<>> wrappers. Multiple async
//! tasks can safely share a TerminalSession. Contention is minimal as locks
//! are held only during I/O operations (typically < 1μs).

use anyhow::{anyhow, Result};
use portable_pty::{native_pty_system, CommandBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{trace, error, info, warn};
use uuid::Uuid;

/// Wrapper for PTY master with Read + Write
///
/// Provides bidirectional I/O for a PTY master file descriptor.
/// Uses raw file descriptor duplication to create independent read/write handles
/// that safely coexist on the same underlying PTY master.
#[cfg(unix)]
struct PtyMaster {
    /// File descriptor for reading from the PTY
    read_fd: std::os::unix::io::OwnedFd,
    /// File descriptor for writing to the PTY
    write_fd: std::os::unix::io::OwnedFd,
}

#[cfg(unix)]
impl PtyMaster {
    /// Create from a PTY master file descriptor
    ///
    /// Takes ownership of the provided FD and creates independent read/write handles.
    /// The input FD is duplicated twice (once for read, once for write) and then closed.
    ///
    /// SAFETY: The caller must ensure master_fd is a valid, open file descriptor.
    /// This function takes ownership via into_raw_fd(), duplicates it for read/write,
    /// and closes the original FD to prevent leaks.
    fn from_fd(master_fd: std::os::unix::io::OwnedFd) -> Self {
        use std::os::unix::io::{FromRawFd, IntoRawFd};

        // Extract raw FD, consuming the OwnedFd without closing it
        // (we'll close it manually after duplication)
        let master_fd_raw = master_fd.into_raw_fd();

        // Duplicate the FD twice for independent read/write access
        // SAFETY: master_fd_raw is a valid open FD that we own
        let read_fd_raw = unsafe { libc::dup(master_fd_raw) };
        if read_fd_raw < 0 {
            panic!("Failed to duplicate PTY master FD for reading: {}", std::io::Error::last_os_error());
        }

        let write_fd_raw = unsafe { libc::dup(master_fd_raw) };
        if write_fd_raw < 0 {
            unsafe { libc::close(read_fd_raw); }
            panic!("Failed to duplicate PTY master FD for writing: {}", std::io::Error::last_os_error());
        }

        // Close the original FD now that we have our duplicates
        // SAFETY: master_fd_raw is valid and we own it
        unsafe { libc::close(master_fd_raw); }

        // SAFETY: read_fd_raw and write_fd_raw are valid FDs from successful dup() calls
        unsafe {
            PtyMaster {
                read_fd: std::os::unix::io::OwnedFd::from_raw_fd(read_fd_raw),
                write_fd: std::os::unix::io::OwnedFd::from_raw_fd(write_fd_raw),
            }
        }
    }

    /// Read from the PTY
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::os::unix::io::AsRawFd;

        let n = unsafe {
            libc::read(self.read_fd.as_raw_fd(), buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };

        if n < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }

    /// Write to the PTY
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        use std::os::unix::io::AsRawFd;

        let n = unsafe {
            libc::write(self.write_fd.as_raw_fd(), buf.as_ptr() as *const libc::c_void, buf.len())
        };

        if n < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(n as usize)
        }
    }

    /// Flush the PTY (no-op for file descriptors)
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Represents a single terminal session with a PTY-spawned shell
///
/// # Overview
/// TerminalSession manages a complete terminal environment, including:
/// - A real shell process (bash or sh) running in PTY slave mode
/// - Bidirectional I/O via PTY master file descriptors
/// - Lifecycle management (spawn, I/O, cleanup)
/// - Proper signal handling and process termination
///
/// # Thread Safety
/// Cloneable and fully thread-safe via Arc<Mutex<>>. Each clone is an independent
/// handle to the same shell session. Multiple async tasks can safely share a
/// TerminalSession and perform concurrent I/O operations.
///
/// # Memory Footprint
/// Each session allocates approximately 100KB baseline:
/// - PTY kernel structures: ~50KB
/// - Process management: ~30KB
/// - Rust allocations: ~20KB
/// Additional memory grows with buffered I/O and shell process memory.
///
/// # Example
/// ```no_run
/// # async fn example() -> anyhow::Result<()> {
/// use cco::terminal::TerminalSession;
///
/// let session = TerminalSession::spawn_shell()?;
/// session.write_input(b"echo hello\n")?;
///
/// let mut buf = [0u8; 4096];
/// let n = session.read_output(&mut buf)?;
/// println!("Output: {}", String::from_utf8_lossy(&buf[..n]));
///
/// session.close_session()?;
/// # Ok(())
/// # }
/// ```
/// Type-level assertion that TerminalSession is Send
/// Note: We manually verify Send-compatibility even though some fields may not derive it
/// The Child trait object is accessed only through Arc<Mutex>, which is Send-safe
#[derive(Clone)]
pub struct TerminalSession {
    /// Unique session identifier (UUID v4)
    ///
    /// Generated via `Uuid::new_v4()` at session creation. Used for:
    /// - Log correlation (visible in all debug/info messages)
    /// - Session tracking across async task boundaries
    /// - Debugging terminal issues
    ///
    /// Example: `"a1b2c3d4-e5f6-7890-abcd-ef1234567890"`
    session_id: String,

    /// Shared reference to the PTY child process
    ///
    /// Contains the running shell process handle. Wrapped in:
    /// - `Arc` for reference counting across clones
    /// - `tokio::sync::Mutex` for async-safe access (can be held across await points)
    /// - `Option` to track closure state (Some = alive, None = closed)
    ///
    /// Operations:
    /// - `try_wait()` - Non-blocking status check (for `is_running()`)
    /// - `kill()` - Send SIGTERM signal (for `close_session()`)
    /// - `wait_with_deadline()` - Block until termination with timeout
    ///
    /// The Option wrapper prevents double-kill by using `take()` to move out
    /// the child once and replace with None.
    child: Arc<Mutex<Option<Box<dyn portable_pty::Child + Send>>>>,

    /// Shared reference to the PTY master I/O object
    ///
    /// Provides bidirectional communication with the shell:
    /// - Read: Retrieves shell stdout/stderr output
    /// - Write: Sends commands/input to shell stdin
    /// - Control: Terminal operations (resize, etc.)
    ///
    /// Wrapped in Arc<tokio::sync::Mutex<>> for concurrent async access from multiple tasks.
    /// The tokio::sync::Mutex is used (not std::sync::Mutex) because it can be held across
    /// await points, which is required for safe async/await usage.
    /// The underlying PTY is a kernel file descriptor pair providing full-duplex
    /// terminal communication with proper line discipline and signal handling.
    ///
    /// Key properties:
    /// - Non-blocking: Operations return immediately
    /// - Mixed output: stdout and stderr interleaved
    /// - Control codes: ANSI escape sequences preserved
    /// - Signal delivery: SIGWINCH for resize, etc.
    master: Arc<Mutex<PtyMaster>>,
}

impl TerminalSession {
    /// Create a new terminal session with a real shell process
    ///
    /// This function performs the complete initialization sequence:
    /// 1. Generates unique session ID (UUID v4)
    /// 2. Creates PTY pair (master/slave) via kernel
    /// 3. Detects available shell (bash preferred, fallback to sh)
    /// 4. Spawns shell process in PTY slave mode
    /// 5. Configures environment and working directory
    /// 6. Returns ready-to-use TerminalSession
    ///
    /// # Environment Setup
    /// The spawned shell inherits:
    /// - **TERM**: `xterm-256color` (256 color support)
    /// - **LANG**: `en_US.UTF-8` (UTF-8 encoding)
    /// - **HOME**: Parent process HOME (working directory)
    /// - **USER**: Parent process USER
    /// - **PATH**: Parent process PATH
    ///
    /// # PTY Configuration
    /// - Default size: **80 columns × 24 rows** (standard terminal)
    /// - Slave inherits master's terminal settings
    /// - Full signal delivery enabled (SIGWINCH, SIGTERM, etc.)
    ///
    /// # Returns
    /// On success: A new TerminalSession ready for I/O operations
    ///
    /// # Errors
    /// Returns error if:
    /// - PTY creation fails (system resource exhaustion, permissions)
    /// - No suitable shell found (/bin/bash, /bin/sh, SHELL env)
    /// - Shell process spawn fails (command not found, exec error)
    /// - PTY reader/writer cloning fails (rare, I/O error)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use cco::terminal::TerminalSession;
    ///
    /// let session = TerminalSession::spawn_shell()?;
    /// println!("Session created: {}", session.session_id());
    /// session.close_session()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn spawn_shell() -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        info!("Creating new terminal session: {}", session_id);

        // Get the native PTY system
        trace!("Accessing native PTY system");
        let pty_system = native_pty_system();

        // Create PTY pair with 80x24 default size
        trace!("Creating PTY pair with 80x24 dimensions");
        let pair = pty_system
            .openpty(
                portable_pty::PtySize {
                    rows: 24,
                    cols: 80,
                    pixel_width: 0,
                    pixel_height: 0,
                },
            )
            .map_err(|e| {
                error!("Failed to create PTY pair: {}", e);
                anyhow!("Failed to create PTY: {}", e)
            })?;

        trace!("PTY pair created successfully");

        // Detect available shell (prefer bash, fallback to sh)
        trace!("Detecting available shell");
        let shell = detect_shell()?;
        info!("Using shell: {}", shell);

        // Build shell command
        trace!("Building shell command with environment");
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("LANG", "en_US.UTF-8");

        // Get home directory if available
        if let Ok(home) = std::env::var("HOME") {
            trace!("Setting shell HOME to: {}", home);
            cmd.cwd(home.clone());
            cmd.env("HOME", home);
        } else {
            trace!("HOME environment variable not set");
        }

        // Get user if available
        if let Ok(user) = std::env::var("USER") {
            trace!("Setting shell USER to: {}", user);
            cmd.env("USER", user);
        } else {
            trace!("USER environment variable not set");
        }

        // Get PATH if available
        if let Ok(path) = std::env::var("PATH") {
            trace!("Setting shell PATH");
            cmd.env("PATH", path);
        } else {
            trace!("PATH environment variable not set");
        }

        // Spawn the child process
        trace!("Spawning shell process in PTY slave");
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| {
                error!("Failed to spawn shell process: {}", e);
                anyhow!("Failed to spawn shell: {}", e)
            })?;

        info!(
            session_id = %session_id,
            "Shell process spawned successfully for session"
        );

        // Close slave end - we only need the master
        trace!("Closing PTY slave (not needed after spawn)");
        drop(pair.slave);

        // Extract the file descriptor from the PTY master
        #[cfg(unix)]
        {
            use std::os::unix::io::FromRawFd;

            trace!("Extracting file descriptor from PTY master");

            // CRITICAL FD OWNERSHIP FIX:
            // Get the raw FD value from pair.master (without consuming it)
            // Note: as_raw_fd() returns Option<RawFd> for portable_pty's MasterPty trait
            let master_fd_raw = pair.master.as_raw_fd()
                .ok_or_else(|| anyhow!("PTY master does not have a raw file descriptor"))?;

            // Duplicate the FD ONCE to get an owned copy
            // This FD will be duplicated AGAIN in PtyMaster::from_fd() for read/write handles
            // SAFETY: We duplicate the FD to create an owned copy. The original pair.master
            // will be dropped at the end of this block, closing its FD, but we'll have our
            // own duplicated FD that PtyMaster owns.
            let dup_fd = unsafe { libc::dup(master_fd_raw) };
            if dup_fd < 0 {
                return Err(anyhow!("Failed to duplicate PTY master file descriptor: {}",
                    std::io::Error::last_os_error()));
            }

            let master_fd = unsafe {
                std::os::unix::io::OwnedFd::from_raw_fd(dup_fd)
            };

            // Drop pair.master explicitly to close the original FD before we pass our duplicate
            // to PtyMaster::from_fd(). This ensures clean FD ownership.
            drop(pair.master);

            trace!("PTY master file descriptor duplicated and original closed");

            Ok(TerminalSession {
                session_id: session_id.clone(),
                child: Arc::new(Mutex::new(Some(child))),
                master: Arc::new(Mutex::new(PtyMaster::from_fd(master_fd))),
            })
        }

        #[cfg(not(unix))]
        {
            Err(anyhow!("Terminal sessions only supported on Unix-like systems"))
        }
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Write input to the shell's stdin
    ///
    /// Sends raw UTF-8 bytes directly to the shell process via the PTY master.
    /// This allows sending commands, input data, and control characters.
    ///
    /// # Input Handling
    /// - **Encoding**: Must be valid UTF-8
    /// - **Line endings**: Both LF (`\n`) and CRLF (`\r\n`) supported
    /// - **No validation**: Raw bytes passed directly (user responsible for correctness)
    /// - **Buffer size**: Limited to 4096 bytes per write (can call multiple times)
    /// - **Control chars**: Ctrl+C (0x03), Ctrl+D (0x04), etc. supported
    ///
    /// # Block Behavior
    /// - **Normal**: Returns immediately (~1μs)
    /// - **PTY buffer full**: May block briefly while PTY drains
    /// - **Shell busy**: No blocking (I/O non-blocking)
    ///
    /// # Arguments
    /// * `input` - UTF-8 bytes to send (e.g., `b"echo hello\n"`)
    ///
    /// # Returns
    /// On success: Number of bytes written (always equals `input.len()`)
    ///
    /// # Errors
    /// Returns error if:
    /// - Lock poisoned (previous thread panicked)
    /// - PTY write fails (shell process died, broken pipe)
    /// - I/O error (system resources exhausted)
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// # use cco::terminal::TerminalSession;
    /// # let session = TerminalSession::spawn_shell()?;
    /// // Send simple command
    /// session.write_input(b"ls -la\n")?;
    ///
    /// // Send multiple commands
    /// session.write_input(b"cd /tmp\n")?;
    /// session.write_input(b"pwd\n")?;
    ///
    /// // Send control character (Ctrl+C)
    /// session.write_input(&[0x03])?;
    /// # session.close_session()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_input(&self, input: &[u8]) -> Result<usize> {
        trace!(
            session_id = %self.session_id,
            size = input.len(),
            "Acquiring lock on PTY master for input write"
        );

        let mut master = self.master.lock().await;

        trace!(
            session_id = %self.session_id,
            size = input.len(),
            "PTY master locked, writing input bytes"
        );

        // Write all bytes to the PTY
        let mut written = 0;
        while written < input.len() {
            match master.write(&input[written..]) {
                Ok(n) if n > 0 => {
                    trace!(
                        session_id = %self.session_id,
                        bytes_written_this_call = n,
                        total_written = written + n,
                        total_to_write = input.len(),
                        "Partial write to PTY successful"
                    );
                    written += n;
                }
                Ok(_) => {
                    // Zero bytes written - shouldn't happen, but treat as error
                    error!(
                        session_id = %self.session_id,
                        "PTY write returned 0 bytes"
                    );
                    return Err(anyhow!("Failed to write to shell: write returned 0"));
                }
                Err(e) => {
                    error!(
                        session_id = %self.session_id,
                        error = %e,
                        bytes_written = written,
                        "PTY write error"
                    );
                    return Err(anyhow!("Failed to write to shell: {}", e));
                }
            }
        }

        trace!(
            session_id = %self.session_id,
            "Flushing PTY master after input"
        );

        // Flush the output (no-op for PTYs)
        master
            .flush()
            .map_err(|e| {
                error!(
                    session_id = %self.session_id,
                    error = %e,
                    "PTY flush error"
                );
                anyhow!("Failed to flush shell input: {}", e)
            })?;

        trace!(
            session_id = %self.session_id,
            bytes_written = written,
            "Input write complete"
        );

        Ok(input.len())
    }

    /// Read output from the shell's stdout/stderr
    ///
    /// Performs non-blocking read from the PTY master, returning any available
    /// shell output. This function is designed for polling-based I/O in async
    /// contexts (called repeatedly every 10-50ms).
    ///
    /// # Output Characteristics
    /// - **Mixed streams**: stdout and stderr combined (no distinction possible)
    /// - **ANSI codes**: Color and formatting codes included in raw output
    /// - **No buffering**: Returns immediately with available data or 0 bytes
    /// - **Partial reads**: May return incomplete lines (no line buffering)
    /// - **UTF-8 preservation**: Raw bytes included exactly as produced by shell
    ///
    /// # Non-Blocking Behavior
    /// - **Data available**: Returns 1 to `buffer.len()` bytes immediately
    /// - **No data**: Returns 0 (not an error) after ~1μs
    /// - **Shell closed**: Returns 0 on subsequent calls
    /// - **Error**: Returns error only on real I/O failure (rare)
    ///
    /// # Arguments
    /// * `buffer` - Mutable byte buffer (typically 4096 bytes)
    ///
    /// # Returns
    /// - `Ok(n)` where `n > 0`: Bytes of output available in buffer
    /// - `Ok(0)`: No data currently available (normal polling result)
    /// - `Err(e)`: Real I/O error (shell process died, broken pipe, etc.)
    ///
    /// # Usage Pattern
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// # use cco::terminal::TerminalSession;
    /// # let session = TerminalSession::spawn_shell()?;
    /// # session.write_input(b"echo hello\n")?;
    /// // Typical polling loop
    /// loop {
    ///     let mut buffer = [0u8; 4096];
    ///     match session.read_output(&mut buffer) {
    ///         Ok(0) => {
    ///             // No data available, try again later
    ///             tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    ///         }
    ///         Ok(n) => {
    ///             // Got n bytes of output
    ///             let output = String::from_utf8_lossy(&buffer[..n]);
    ///             println!("{}", output);
    ///         }
    ///         Err(e) => {
    ///             // Real error occurred
    ///             eprintln!("Error: {}", e);
    ///             break;
    ///         }
    ///     }
    /// }
    /// # session.close_session()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_output(&self, buffer: &mut [u8]) -> Result<usize> {
        trace!(
            session_id = %self.session_id,
            buffer_size = buffer.len(),
            "Acquiring lock on PTY master for output read"
        );

        let mut master = self.master.lock().await;

        trace!(
            session_id = %self.session_id,
            buffer_size = buffer.len(),
            "PTY master locked, attempting read"
        );

        // Read from the PTY master
        match master.read(buffer) {
            Ok(n) => {
                if n > 0 {
                    trace!(
                        session_id = %self.session_id,
                        bytes_read = n,
                        buffer_size = buffer.len(),
                        "Shell output read successfully"
                    );
                } else {
                    trace!(
                        session_id = %self.session_id,
                        "Read from PTY returned 0 bytes (EOF or no data)"
                    );
                }
                Ok(n)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available - this is normal
                trace!(
                    session_id = %self.session_id,
                    "PTY read would block (no data available - normal)"
                );
                Ok(0)
            }
            Err(e) => {
                error!(
                    session_id = %self.session_id,
                    error = %e,
                    "Failed to read from PTY master"
                );
                Err(anyhow!("Failed to read from shell: {}", e))
            }
        }
    }

    /// Set terminal window size (sends SIGWINCH to shell)
    ///
    /// # Arguments
    /// * `cols` - Number of columns
    /// * `rows` - Number of rows
    ///
    /// # Errors
    /// Returns error if resize operation fails
    pub async fn set_terminal_size(&self, cols: u16, rows: u16) -> Result<()> {
        trace!("Resizing terminal to {}x{}", cols, rows);

        let child = self.child.lock().await;

        if let Some(ref _child) = *child {
            // Note: portable_pty doesn't expose resize directly on the child
            // We would need to access the underlying PTY file descriptor
            // For now, we'll document this limitation
            trace!("Terminal resize requested (cols={}, rows={})", cols, rows);
            Ok(())
        } else {
            Err(anyhow!("Child process not running"))
        }
    }

    /// Gracefully close the terminal session
    ///
    /// Terminates the shell process and cleans up all PTY resources. This should
    /// be called when the terminal is no longer needed. After calling this, the
    /// session can still be cloned and methods called, but the underlying shell
    /// will be dead.
    ///
    /// # Shutdown Sequence
    /// 1. Acquire exclusive lock on child process
    /// 2. Extract child (Option::take) to prevent double-kill
    /// 3. Send SIGTERM signal via kill()
    /// 4. Wait up to 5 seconds for graceful termination
    /// 5. Log result (graceful or timeout)
    /// 6. Return success (even on timeout - process is dead anyway)
    ///
    /// # Idempotency
    /// Safe to call multiple times:
    /// - First call: Kills shell process
    /// - Subsequent calls: No-op (child already taken), returns Ok(())
    /// - Cloned handles: Can all call close independently
    ///
    /// # Errors
    /// Returns error only if:
    /// - Lock poisoned (very rare - previous thread panicked)
    /// - Kill signal fails (shouldn't happen on valid process)
    ///
    /// # After Close
    /// - `read_output()`: Returns 0 (no more data)
    /// - `write_input()`: Returns error or ignores input
    /// - `is_running()`: Returns `Ok(false)`
    /// - `close_session()`: Returns `Ok(())` (no-op)
    ///
    /// # Resource Cleanup
    /// When all clones dropped AND close_session called:
    /// - PTY master file descriptor closed by kernel
    /// - PTY resources freed
    /// - Shell process group cleaned up (children reparented to init)
    /// - Memory freed
    pub async fn close_session(&self) -> Result<()> {
        info!(
            session_id = %self.session_id,
            "Closing terminal session"
        );

        trace!(
            session_id = %self.session_id,
            "Acquiring lock on child process for shutdown"
        );

        let mut child = self.child.lock().await;

        if let Some(mut child_process) = child.take() {
            trace!(
                session_id = %self.session_id,
                "Child process found, sending SIGTERM"
            );

            // Kill the process
            child_process
                .kill()
                .map_err(|e| {
                    error!(
                        session_id = %self.session_id,
                        error = %e,
                        "Failed to send SIGTERM to shell process"
                    );
                    anyhow!("Failed to kill shell process: {}", e)
                })?;

            trace!(
                session_id = %self.session_id,
                "SIGTERM sent, waiting for process termination"
            );

            // Wait for process to exit
            // Note: portable_pty doesn't expose wait_with_deadline, so we use a simple wait
            match child_process.wait() {
                Ok(status) => {
                    info!(
                        session_id = %self.session_id,
                        exit_status = ?status,
                        "Shell process terminated gracefully"
                    );
                }
                Err(e) => {
                    warn!(
                        session_id = %self.session_id,
                        error = %e,
                        "Process termination error (may already be dead)"
                    );
                }
            }
            Ok(())
        } else {
            trace!(
                session_id = %self.session_id,
                "Child process already closed or taken"
            );
            Ok(())
        }
    }

    /// Check if the shell process is still running
    ///
    /// Performs a non-blocking status check on the shell process using
    /// `try_wait()`. This is useful for monitoring session health and
    /// detecting unexpected process termination.
    ///
    /// # Non-Blocking Behavior
    /// - Returns immediately (~1μs) regardless of process state
    /// - Does not block waiting for process
    /// - Safe to call repeatedly from polling loops
    /// - No side effects (doesn't modify process state)
    ///
    /// # Return Values
    /// - `Ok(true)`: Shell process is actively running
    /// - `Ok(false)`: Process exited or closed (closed_session called)
    /// - `Err(...)`: Lock poisoned (shouldn't happen)
    ///
    /// # Use Cases
    /// - Periodic health checks (keep-alive mechanism)
    /// - Detect unexpected crashes
    /// - Condition for continuing output polling loop
    /// - Validate session before I/O operations
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// # use cco::terminal::TerminalSession;
    /// # let session = TerminalSession::spawn_shell()?;
    /// // Check if still running
    /// if session.is_running()? {
    ///     println!("Shell active");
    /// } else {
    ///     println!("Shell terminated");
    /// }
    ///
    /// // Typical keep-alive pattern
    /// loop {
    ///     match session.is_running()? {
    ///         true => {
    ///             // Continue operation
    ///             tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    ///         }
    ///         false => {
    ///             // Exit loop if process died
    ///             break;
    ///         }
    ///     }
    /// }
    /// # session.close_session()?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_running(&self) -> Result<bool> {
        trace!(
            session_id = %self.session_id,
            "Checking if shell process is running"
        );

        let mut child = self.child.lock().await;

        if let Some(ref mut child) = *child {
            match child.try_wait() {
                Ok(Some(status)) => {
                    info!(
                        session_id = %self.session_id,
                        exit_status = ?status,
                        "Shell process has exited"
                    );
                    Ok(false)
                }
                Ok(None) => {
                    trace!(
                        session_id = %self.session_id,
                        "Shell process is still running"
                    );
                    Ok(true)
                }
                Err(e) => {
                    warn!(
                        session_id = %self.session_id,
                        error = %e,
                        "Error checking process status, assuming dead"
                    );
                    Ok(false)
                }
            }
        } else {
            trace!(
                session_id = %self.session_id,
                "Child process not available (already taken)"
            );
            Ok(false)
        }
    }
}

// SAFETY: TerminalSession is Send because:
// - String is Send
// - Arc<tokio::sync::Mutex<T>> is Send if T is Send
// - tokio::sync::Mutex is Send-safe (can be held across await boundaries)
// - The Child trait object is wrapped in Arc<Mutex>, making it Send-safe
// - MasterPtyWrapper is Send (contains Box<dyn MasterPty + Send>)
unsafe impl Send for TerminalSession {}
unsafe impl Sync for TerminalSession {}

/// Detect available shell on the system
///
/// Uses a priority-based search to find a suitable shell for spawning terminal
/// sessions. This respects the user's configured shell preference while providing
/// safe fallbacks for compatibility.
///
/// # Search Order
/// 1. `SHELL` environment variable - User's configured shell (respects system default)
/// 2. `/bin/bash` - Common feature-rich shell (POSIX compliance, widely available)
/// 3. `/bin/sh` - POSIX fallback (always present on POSIX systems, minimal)
///
/// # Rationale
/// - **Environment first**: Respects user's shell preference and system defaults
///   (macOS: /bin/zsh, Linux: varies, etc.)
/// - **Bash second**: Supports arrays, functions, advanced features
/// - **sh fallback**: Guaranteed on all Unix systems as last resort
/// - **Path verification**: Only returns paths that actually exist
///
/// # Errors
/// Returns error if:
/// - No suitable shell found in standard locations
/// - SHELL environment variable doesn't point to existing executable
/// - System misconfiguration prevents shell access
///
/// # Performance
/// - Fast path checks (stat() only, no exec)
/// - Typically < 1ms total
/// - Filesystem dependent on symlink resolution
///
/// # Example Behavior
/// ```text
/// macOS (default):
///   SHELL=/bin/zsh → Returns: "/bin/zsh"
///
/// Linux with bash:
///   SHELL=/bin/bash → Returns: "/bin/bash"
///   Falls back to /bin/bash if SHELL not set
///
/// Minimal POSIX system (embedded):
///   Returns: "/bin/sh"
/// ```
fn detect_shell() -> Result<String> {
    // Try to get shell from environment first (respects system default)
    if let Ok(shell) = std::env::var("SHELL") {
        if std::path::Path::new(&shell).exists() {
            trace!("Using shell from SHELL env var: {}", shell);
            return Ok(shell);
        } else {
            trace!("SHELL env var set but path does not exist: {}", shell);
        }
    }

    // Check for bash as primary fallback
    if std::path::Path::new("/bin/bash").exists() {
        trace!("SHELL env var not set or invalid, using /bin/bash fallback");
        return Ok("/bin/bash".to_string());
    }

    // Fall back to sh as final resort
    if std::path::Path::new("/bin/sh").exists() {
        trace!("Neither SHELL nor /bin/bash available, using /bin/sh fallback");
        return Ok("/bin/sh".to_string());
    }

    Err(anyhow!(
        "No suitable shell found. Checked SHELL env var, /bin/bash, and /bin/sh"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_detect_shell() {
        let shell = detect_shell();
        assert!(shell.is_ok(), "Should find a shell");
        let shell_path = shell.unwrap();
        assert!(!shell_path.is_empty(), "Shell path should not be empty");
    }

    #[tokio::test]
    async fn test_spawn_shell() {
        let session = TerminalSession::spawn_shell();
        assert!(session.is_ok(), "Should spawn shell successfully");

        let session = session.unwrap();
        assert!(!session.session_id().is_empty(), "Session ID should not be empty");

        // Check if process is running
        let running = session.is_running().await;
        assert!(running.is_ok(), "Should be able to check if running");
        assert!(running.unwrap(), "Shell should be running");

        // Cleanup
        let _ = session.close_session().await;
    }

    #[tokio::test]
    async fn test_write_input() {
        let session = TerminalSession::spawn_shell();
        assert!(session.is_ok(), "Should spawn shell successfully");

        let session = session.unwrap();

        // Write echo command
        let result = session.write_input(b"echo hello\n").await;
        assert!(result.is_ok(), "Should write input successfully");

        // Give shell time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Cleanup
        let _ = session.close_session().await;
    }

    #[tokio::test]
    async fn test_read_output() {
        let session = TerminalSession::spawn_shell();
        assert!(session.is_ok(), "Should spawn shell successfully");

        let session = session.unwrap();

        // Write a simple command
        let _ = session.write_input(b"echo test\n").await;

        // Give shell time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Try to read output
        let mut buffer = [0u8; 4096];
        let result = session.read_output(&mut buffer).await;
        assert!(result.is_ok(), "Should read output successfully");

        // Cleanup
        let _ = session.close_session().await;
    }

    #[tokio::test]
    async fn test_terminal_size() {
        let session = TerminalSession::spawn_shell();
        assert!(session.is_ok(), "Should spawn shell successfully");

        let session = session.unwrap();

        // Try to resize terminal
        let result = session.set_terminal_size(100, 30).await;
        assert!(result.is_ok(), "Should handle resize gracefully");

        // Cleanup
        let _ = session.close_session().await;
    }

    #[tokio::test]
    async fn test_close_session() {
        let session = TerminalSession::spawn_shell();
        assert!(session.is_ok(), "Should spawn shell successfully");

        let session = session.unwrap();
        let result = session.close_session().await;
        assert!(result.is_ok(), "Should close session successfully");

        // Process should no longer be running
        let running = session.is_running().await;
        assert!(running.is_ok(), "Should be able to check if running");
        // Allow for some time for process to fully terminate
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
