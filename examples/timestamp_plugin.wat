;; Example WASM Plugin: Timestamp Transformer
;;
;; This plugin adds a timestamp prefix to terminal output.
;; It demonstrates how to write a WASM plugin for warp-foss-clone.
;;
;; Plugin API:
;;   - Export `memory` - At least 1 page (64KB) of memory
;;   - Export `plugin_id` - Returns pointer to null-terminated string with plugin ID
;;   - Export `plugin_name` - Returns pointer to null-terminated string with plugin name (optional)
;;   - Export `plugin_version` - Returns pointer to version string (optional)
;;   - Export `plugin_author` - Returns pointer to author string (optional)
;;   - Export `plugin_description` - Returns pointer to description string (optional)
;;   - Export `on_input(ptr, len) -> len` - Called on user input (optional)
;;   - Export `on_output(ptr, len) -> len` - Called on terminal output (optional)
;;
;; Hook functions:
;;   - Parameters: (ptr: i32, len: i32) - Pointer to data and length
;;   - Returns: i32 - Length of modified data (0 = no modification)
;;   - Modified data should be written starting at offset 0 in memory
;;
;; Security: Plugins are sandboxed with NO access to:
;;   - Filesystem
;;   - Network
;;   - Environment variables
;;   - System calls
;;
;; To compile: wat2wasm timestamp_plugin.wat -o timestamp_plugin.wasm
;; Or use: wat --parse timestamp_plugin.wat > timestamp_plugin.wasm (via wabt)

(module
    ;; Memory export - 2 pages (128KB) for data processing
    (memory (export "memory") 2 10)

    ;; ============================================
    ;; Plugin Metadata
    ;; ============================================

    ;; Plugin ID - unique identifier
    (func (export "plugin_id") (result i32)
        i32.const 0
    )

    ;; Plugin name for display
    (func (export "plugin_name") (result i32)
        i32.const 32
    )

    ;; Version string
    (func (export "plugin_version") (result i32)
        i32.const 64
    )

    ;; Author information
    (func (export "plugin_author") (result i32)
        i32.const 72
    )

    ;; Description
    (func (export "plugin_description") (result i32)
        i32.const 96
    )

    ;; ============================================
    ;; Hook Functions
    ;; ============================================

    ;; on_input: Pass through input unchanged
    ;; Returns 0 to indicate no modification
    (func (export "on_input") (param $ptr i32) (param $len i32) (result i32)
        ;; Don't modify input
        i32.const 0
    )

    ;; on_output: Add "[timestamp] " prefix to output
    ;; This is a simple example - in reality you'd get the actual timestamp
    (func (export "on_output") (param $ptr i32) (param $len i32) (result i32)
        (local $i i32)
        (local $dest i32)

        ;; Write "[TS] " prefix at position 0
        ;; '['
        (i32.store8 (i32.const 0) (i32.const 91))
        ;; 'T'
        (i32.store8 (i32.const 1) (i32.const 84))
        ;; 'S'
        (i32.store8 (i32.const 2) (i32.const 83))
        ;; ']'
        (i32.store8 (i32.const 3) (i32.const 93))
        ;; ' '
        (i32.store8 (i32.const 4) (i32.const 32))

        ;; Copy original data after prefix (offset 5)
        (local.set $i (i32.const 0))
        (local.set $dest (i32.const 5))

        (block $done
            (loop $copy
                ;; Check if we've copied all bytes
                (br_if $done (i32.ge_u (local.get $i) (local.get $len)))

                ;; Copy byte: memory[dest + i] = memory[ptr + i]
                (i32.store8
                    (i32.add (local.get $dest) (local.get $i))
                    (i32.load8_u (i32.add (local.get $ptr) (local.get $i)))
                )

                ;; Increment counter
                (local.set $i (i32.add (local.get $i) (i32.const 1)))
                (br $copy)
            )
        )

        ;; Return new length: prefix (5) + original length
        (i32.add (local.get $len) (i32.const 5))
    )

    ;; ============================================
    ;; Data Section - Strings
    ;; ============================================
    (data (i32.const 0) "timestamp-plugin\00")
    (data (i32.const 32) "Timestamp Plugin\00")
    (data (i32.const 64) "1.0.0\00")
    (data (i32.const 72) "Warp Team\00")
    (data (i32.const 96) "Adds timestamp prefix to terminal output\00")
)
