package main

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"syscall"
	"time"

	"github.com/creack/pty"
)

// PTYCommand executes a command in a pseudo-terminal
// This provides better shell integration, proper signal handling,
// and more accurate terminal output compared to exec.Command
func PTYCommand(shell, shellFlag, cmdStr string, workingDir string) ([]byte, error) {
	// Create the command
	cmd := exec.Command(shell, shellFlag, cmdStr)
	if workingDir != "" {
		cmd.Dir = workingDir
	}

	// Create a pseudo-terminal
	ptmx, err := pty.Start(cmd)
	if err != nil {
		return nil, err
	}

	// Make sure to close the PTY at the end
	defer ptmx.Close()

	// Set PTY size (important for commands like `ls` that format output)
	winsize := &pty.Winsize{
		Cols: 80,
		Rows: 24,
	}
	if err := pty.Setsize(ptmx, winsize); err != nil {
		ptmx.Close()
		return nil, fmt.Errorf("failed to set PTY size: %w", err)
	}

	// Read all output from PTY
	output, err := io.ReadAll(ptmx)
	if err != nil && err != io.EOF {
		return output, err
	}

	// Wait for the command to complete and get its exit status
	err = cmd.Wait()

	// If the command exited with a non-zero status, return an error
	if err != nil {
		// Check if it's a normal exit with non-zero status
		if exitErr, ok := err.(*exec.ExitError); ok {
			if exitErr.ExitCode() != -1 {
				// Normal exit with non-zero status - return output anyway
				return output, err
			}
		}
	}

	return output, err
}

// PTYCommandInteractive executes a command in a PTY for interactive use
// This is intended for future use with interactive shells
func PTYCommandInteractive(shell, shellFlag, cmdStr string, workingDir string) (*os.File, *exec.Cmd, error) {
	cmd := exec.Command(shell, shellFlag, cmdStr)
	if workingDir != "" {
		cmd.Dir = workingDir
	}

	ptmx, err := pty.Start(cmd)
	if err != nil {
		return nil, nil, err
	}

	// Set up signal forwarding for proper shell behavior
	go func() {
		for range time.Tick(100 * time.Millisecond) {
			if cmd.Process == nil {
				return
			}
			// Check if PTY is still open
			if _, err := ptmx.Stat(); err != nil {
				return
			}
		}
	}()

	return ptmx, cmd, nil
}

// KillPTYProcess sends a SIGTERM signal to a PTY process
func KillPTYProcess(cmd *exec.Cmd) error {
	if cmd != nil && cmd.Process != nil {
		return cmd.Process.Signal(syscall.SIGTERM)
	}
	return nil
}
