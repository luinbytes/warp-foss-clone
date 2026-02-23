//! PTY (Pseudo-Terminal) management using portable-pty
//!
//! This module provides cross-platform PTY spawning and I/O operations
//! for the terminal emulator.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use thiserror::Error;

/// Errors that can occur during PTY operations
#[derive(Error, Debug)]
pub enum PtyError {
    #[error("Failed to create PTY: {0}")]
    CreationFailed(String),

    #[error("Failed to spawn shell: {0}")]
    SpawnFailed(String),

    #[error("Failed to read from PTY: {0}")]
    ReadError(String),

    #[error("Failed to write to PTY: {0}")]
    WriteError(String),

    #[error("Failed to resize PTY: {0}")]
    ResizeError(String),

    #[error("PTY not initialized")]
    NotInitialized,

    #[error("Shell not found in environment")]
    ShellNotFound,
}

/// Result type for PTY operations
pub type PtyResult<T> = Result<T, PtyError>;

/// Configuration for PTY session
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Shell command to spawn (defaults to $SHELL or /bin/sh)
    pub shell: Option<String>,
    /// Initial terminal width in columns
    pub cols: u16,
    /// Initial terminal height in rows
    pub rows: u16,
    /// Working directory for the shell
    pub working_dir: Option<String>,
    /// Environment variables to set
    pub env: Vec<(String, String)>,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            shell: None,
            cols: 80,
            rows: 24,
            working_dir: None,
            env: Vec::new(),
        }
    }
}

/// Writer handle for sending input to the PTY
pub struct PtyWriter {
    writer: Box<dyn Write + Send>,
}

impl PtyWriter {
    fn new(writer: Box<dyn Write + Send>) -> Self {
        Self { writer }
    }

    /// Write data to the PTY (sends input to the shell)
    pub fn write(&mut self, data: &[u8]) -> PtyResult<usize> {
        self.writer
            .write_all(data)
            .map(|_| data.len())
            .map_err(|e| PtyError::WriteError(e.to_string()))
    }

    /// Write a string to the PTY
    pub fn write_str(&mut self, s: &str) -> PtyResult<usize> {
        self.write(s.as_bytes())
    }

    /// Flush the write buffer
    pub fn flush(&mut self) -> PtyResult<()> {
        self.writer
            .flush()
            .map_err(|e| PtyError::WriteError(e.to_string()))
    }
}

/// Reader handle for receiving output from the PTY
pub struct PtyReader {
    reader: Box<dyn Read + Send>,
    /// Buffer for batched reads
    read_buffer: Vec<u8>,
    /// Position in read_buffer
    read_pos: usize,
}

impl PtyReader {
    fn new(reader: Box<dyn Read + Send>) -> Self {
        Self {
            reader,
            read_buffer: Vec::with_capacity(8192),
            read_pos: 0,
        }
    }

    /// Read data from the PTY (receives output from the shell)
    pub fn read(&mut self, buf: &mut [u8]) -> PtyResult<usize> {
        self.reader
            .read(buf)
            .map_err(|e| PtyError::ReadError(e.to_string()))
    }

    /// Read all available data into a vector
    pub fn read_available(&mut self, buf_size: usize) -> PtyResult<Vec<u8>> {
        let mut buffer = vec![0u8; buf_size];
        let n = self.read(&mut buffer)?;
        buffer.truncate(n);
        Ok(buffer)
    }

    /// Read and accumulate data into the internal buffer for batched processing.
    ///
    /// This reads available data from the PTY and stores it in an internal buffer.
    /// Use `take_batch()` to retrieve the accumulated data.
    pub fn read_batch(&mut self) -> PtyResult<usize> {
        let mut temp_buf = [0u8; 4096];
        match self.read(&mut temp_buf) {
            Ok(0) => Ok(0), // EOF
            Ok(n) => {
                // Append to internal buffer
                self.read_buffer.extend_from_slice(&temp_buf[..n]);
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }

    /// Take all accumulated batched data and clear the internal buffer.
    ///
    /// Returns the accumulated bytes and clears the buffer for the next batch.
    pub fn take_batch(&mut self) -> Vec<u8> {
        if self.read_pos > 0 {
            // Shift remaining data to front
            let remaining = self.read_buffer.split_off(self.read_pos);
            let result = std::mem::replace(&mut self.read_buffer, remaining);
            self.read_pos = 0;
            result
        } else {
            std::mem::take(&mut self.read_buffer)
        }
    }

    /// Check if there is accumulated batched data available.
    pub fn has_batched_data(&self) -> bool {
        self.read_pos < self.read_buffer.len()
    }

    /// Get the amount of batched data currently accumulated.
    pub fn batched_len(&self) -> usize {
        self.read_buffer.len() - self.read_pos
    }

    /// Clear the internal batch buffer without processing.
    pub fn clear_batch(&mut self) {
        self.read_buffer.clear();
        self.read_pos = 0;
    }
}

/// A PTY session that manages the pseudo-terminal lifecycle
pub struct PtySession {
    /// The PTY pair (primary + replica)
    pair: PtyPair,
    /// Writer for sending input to the shell
    writer: PtyWriter,
    /// Reader for receiving output from the shell (thread-safe)
    reader: Arc<Mutex<PtyReader>>,
    /// The spawned child process handle
    child: Box<dyn portable_pty::Child + Send + Sync>,
    /// Current terminal size
    size: PtySize,
}

impl PtySession {
    /// Get the default shell for the current environment
    fn get_default_shell() -> PtyResult<String> {
        // Try $SHELL environment variable first
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.is_empty() {
                return Ok(shell);
            }
        }

        // Fall back to /bin/sh (should exist on all Unix systems)
        if std::path::Path::new("/bin/sh").exists() {
            return Ok("/bin/sh".to_string());
        }

        // On Windows, try cmd.exe
        #[cfg(windows)]
        {
            return Ok("cmd.exe".to_string());
        }

        Err(PtyError::ShellNotFound)
    }

    /// Spawn a new shell process in a PTY with the given configuration
    pub fn spawn(config: PtyConfig) -> PtyResult<Self> {
        // Get the native PTY system
        let pty_system = native_pty_system();

        // Determine the shell to use
        let shell = match config.shell {
            Some(s) => s,
            None => Self::get_default_shell()?,
        };

        // Create the PTY size
        let size = PtySize {
            rows: config.rows,
            cols: config.cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        // Open a new PTY pair
        let pair = pty_system
            .openpty(size)
            .map_err(|e| PtyError::CreationFailed(e.to_string()))?;

        // Build the shell command
        let mut cmd = CommandBuilder::new(&shell);

        // Set working directory if specified
        if let Some(ref dir) = config.working_dir {
            cmd.cwd(dir);
        }

        // Set environment variables
        for (key, value) in config.env {
            cmd.env(&key, &value);
        }

        // Set TERM environment variable for proper terminal capabilities
        cmd.env("TERM", "xterm-256color");

        // Spawn the child process in the PTY
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        // Create reader and writer handles
        let writer = PtyWriter::new(
            pair.master
                .take_writer()
                .map_err(|e| PtyError::CreationFailed(e.to_string()))?,
        );

        let reader = PtyReader::new(
            pair.master
                .try_clone_reader()
                .map_err(|e| PtyError::CreationFailed(e.to_string()))?,
        );

        Ok(Self {
            pair,
            writer,
            reader: Arc::new(Mutex::new(reader)),
            size,
            child,
        })
    }

    /// Spawn a new shell with default configuration
    pub fn spawn_shell() -> PtyResult<Self> {
        Self::spawn(PtyConfig::default())
    }

    /// Spawn a new shell with custom size
    pub fn spawn_with_size(cols: u16, rows: u16) -> PtyResult<Self> {
        let config = PtyConfig {
            cols,
            rows,
            ..Default::default()
        };
        Self::spawn(config)
    }

    /// Write data to the PTY (send input to shell)
    pub fn write(&mut self, data: &[u8]) -> PtyResult<usize> {
        self.writer.write(data)
    }

    /// Write a string to the PTY
    pub fn write_str(&mut self, s: &str) -> PtyResult<usize> {
        self.writer.write_str(s)
    }

    /// Read data from the PTY (receive output from shell)
    pub fn read(&self, buf: &mut [u8]) -> PtyResult<usize> {
        let mut reader = self
            .reader
            .lock()
            .map_err(|_| PtyError::ReadError("Reader lock poisoned".to_string()))?;
        reader.read(buf)
    }

    /// Resize the PTY to new dimensions
    pub fn resize(&mut self, cols: u16, rows: u16) -> PtyResult<()> {
        let new_size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        self.pair
            .master
            .resize(new_size)
            .map_err(|e| PtyError::ResizeError(e.to_string()))?;

        self.size = new_size;
        Ok(())
    }

    /// Get the current terminal size
    pub fn size(&self) -> (u16, u16) {
        (self.size.cols, self.size.rows)
    }

    /// Check if the child process is still running
    pub fn is_alive(&mut self) -> PtyResult<bool> {
        // Try to get the exit status without blocking
        match self.child.try_wait() {
            Ok(Some(_status)) => Ok(false), // Process has exited
            Ok(None) => Ok(true),           // Still running
            Err(e) => Err(PtyError::ReadError(e.to_string())),
        }
    }

    /// Wait for the child process to exit and return its status
    pub fn wait(&mut self) -> PtyResult<portable_pty::ExitStatus> {
        self.child
            .wait()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))
    }

    /// Get a clone of the reader for use in another thread
    pub fn reader_clone(&self) -> Arc<Mutex<PtyReader>> {
        Arc::clone(&self.reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_shell() {
        let session = PtySession::spawn_shell();
        assert!(session.is_ok(), "Should be able to spawn a shell");
    }

    #[test]
    fn test_write_and_read() {
        let mut session = PtySession::spawn_shell().expect("Should spawn shell");

        // Send a simple command
        let write_result = session.write_str("echo hello\n");
        assert!(write_result.is_ok(), "Should be able to write to PTY");

        // Give the shell time to process
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Read output
        let mut buf = [0u8; 1024];
        let read_result = session.read(&mut buf);
        assert!(read_result.is_ok(), "Should be able to read from PTY");
    }

    #[test]
    fn test_resize() {
        let mut session = PtySession::spawn_shell().expect("Should spawn shell");

        // Resize to 120x40
        let resize_result = session.resize(120, 40);
        assert!(resize_result.is_ok(), "Should be able to resize PTY");

        let (cols, rows) = session.size();
        assert_eq!(cols, 120);
        assert_eq!(rows, 40);
    }
}
