//! Integration test for terminal emulator
//!
//! Tests the full pipeline: PTY → Parser → Grid

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use warp_foss::terminal::grid::TerminalGrid;
use warp_foss::terminal::parser::TerminalParser;
use warp_foss::terminal::pty::{PtyConfig, PtySession};

/// Test that spawning a PTY and writing to it works
#[test]
fn test_pty_spawn_and_write() {
    let config = PtyConfig {
        cols: 80,
        rows: 24,
        shell: None, // Use default shell
        working_dir: None,
        env: vec![],
    };
    
    let pty = PtySession::spawn(config).expect("Failed to spawn PTY");
    let pty = Arc::new(Mutex::new(pty));
    
    // Give PTY time to initialize
    thread::sleep(Duration::from_millis(100));
    
    // Write a simple command
    {
        let mut session = pty.lock().unwrap();
        session.write(b"echo hello\n").expect("Failed to write to PTY");
    }
    
    // Give command time to execute
    thread::sleep(Duration::from_millis(200));
    
    // Read output using reader_clone
    let reader = pty.lock().unwrap().reader_clone();
    let mut buf = [0u8; 4096];
    let output = {
        let mut reader = reader.lock().unwrap();
        reader.read(&mut buf).expect("Failed to read from PTY")
    };
    
    // Output should contain "hello"
    let output_str = String::from_utf8_lossy(&buf[..output]);
    assert!(output_str.contains("hello"), "Expected output to contain 'hello', got: {}", output_str);
}

/// Test the full pipeline: PTY → Parser → Grid
#[test]
fn test_full_pipeline() {
    let config = PtyConfig {
        cols: 80,
        rows: 24,
        shell: None,
        working_dir: None,
        env: vec![],
    };
    
    let pty = PtySession::spawn(config).expect("Failed to spawn PTY");
    let pty = Arc::new(Mutex::new(pty));
    
    let mut grid = TerminalGrid::new();
    let mut parser = TerminalParser::new();
    
    // Give PTY time to initialize
    thread::sleep(Duration::from_millis(100));
    
    // Write a command that produces simple output
    {
        let mut session = pty.lock().unwrap();
        session.write(b"printf 'test123'\n").expect("Failed to write to PTY");
    }
    
    // Read and process output
    thread::sleep(Duration::from_millis(300));
    
    let reader = pty.lock().unwrap().reader_clone();
    let mut buf = [0u8; 4096];
    let n = {
        let mut reader = reader.lock().unwrap();
        reader.read(&mut buf).expect("Failed to read from PTY")
    };
    
    // Process through parser to grid
    parser.parse_bytes_with_output(&buf[..n], &mut grid);
    
    // Grid should contain "test123" somewhere
    let grid_content = grid_to_string(&grid);
    assert!(
        grid_content.contains("test123"),
        "Expected grid to contain 'test123', got:\n{}",
        grid_content
    );
}

/// Test keyboard input flows through to PTY
#[test]
fn test_keyboard_to_pty() {
    let config = PtyConfig {
        cols: 80,
        rows: 24,
        shell: None,
        working_dir: None,
        env: vec![],
    };
    
    let pty = PtySession::spawn(config).expect("Failed to spawn PTY");
    let pty = Arc::new(Mutex::new(pty));
    
    // Give PTY time to initialize
    thread::sleep(Duration::from_millis(100));
    
    // Simulate typing "ls\n"
    {
        let mut session = pty.lock().unwrap();
        session.write(b"ls\n").expect("Failed to write to PTY");
    }
    
    // Give command time to execute
    thread::sleep(Duration::from_millis(300));
    
    // Read output
    let reader = pty.lock().unwrap().reader_clone();
    let mut buf = [0u8; 4096];
    let n = {
        let mut reader = reader.lock().unwrap();
        reader.read(&mut buf).expect("Failed to read from PTY")
    };
    
    // Output should not be empty (ls should list something)
    assert!(n > 0, "Expected some output from 'ls' command");
}

/// Helper: Convert grid to string for assertions
fn grid_to_string(grid: &TerminalGrid) -> String {
    let mut result = String::new();
    for row in 0..grid.rows() {
        for col in 0..grid.cols() {
            if let Some(cell) = grid.get_cell(row, col) {
                result.push(cell.char);
            } else {
                result.push(' ');
            }
        }
        result.push('\n');
    }
    result
}
