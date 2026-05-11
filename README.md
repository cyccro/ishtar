# Ishtarx

A terminal-based text editor written in Rust using the Ratatui library.

## Description

Ishtarx is a modal text editor inspired by vim, featuring multiple panes, syntax highlighting (via tree-sitter), and a command interface. The editor is designed to be lightweight, fast, and customizable.

## Features

- Modal editing (Command, Modify, Selection modes)
- Split panes (horizontal and vertical)
- File manager with search capabilities
- System clipboard integration
- Keybind handling and customization
- Syntax highlighting via tree-sitter
- Command interpreter for executing shell commands
- Logger for debugging and information display

## Project Structure

```
.
├── Cargo.toml          # Project manifest and dependencies
├── Cargo.lock          # Dependency lock file
├── src/
│   ├── main.rs         # Application entry point
│   ├── helpers/        # Utility functions and types
│   │   ├── functions.rs # Helper functions (terminal size, char handling)
│   │   ├── types.rs     # Type definitions
│   │   ├── vec2.rs      # 2D vector implementation
│   │   ├── terminal_line.rs # Terminal line handling
│   │   ├── file_tree.rs # File tree data structure
│   │   └── mod.rs       # Helpers module exports
│   └── ishtar/         # Main editor components
│       ├── enums.rs     # Editor modes (Cmd, Modify, Selection)
│       ├── errors.rs    # Error handling
│       ├── logger.rs    # Logging functionality
│       ├── mod.rs       # Core Ishtar struct and implementation
│       ├── widget_manager.rs # Widget management system
│       └── widgets/     # UI components
│           ├── clipboard.rs     # System clipboard integration
│           ├── command_interpreter.rs # Command line interface
│           ├── file_manager.rs  # File browsing and searching
│           ├── keybind_handler.rs # Keyboard input handling
│           ├── popup.rs         # Popup windows
│           ├── text_area.rs     # Text editing area
│           ├── writeable_area.rs # Multi-pane text area container
│           └── mod.rs           # Widget traits and exports
└── tmp/                # Temporary files (logs, etc.)
```

## Dependencies

- `ratatui` - Terminal UI framework
- `tree-sitter` & `tree-sitter-highlight` - Syntax highlighting
- `tree-sitter-javascript` - JavaScript grammar for tree-sitter
- `anyhow` - Error handling
- `chrono` - Date and time handling
- `copypasta` - Clipboard access
- `unicode-normalization` - Unicode text processing
- `gapbuf` - Gap buffer for efficient text editing
- `downcast-rs` - Trait downcasting
- `tachyonfx` - Terminal effects
- `isht` - Local dependency (appears to be a configuration library)

## Build Instructions

```bash
# Clone the repository
git clone <repository-url>
cd ishtar

# Build the project
cargo build --release

# Run the editor
cargo run --release
```

## Configuration

The editor looks for a configuration file at `~/.config/ishtar/config.isht`. If not found, it uses default configuration.

## Architecture Overview

Ishtarx follows a modular architecture centered around:

1. **Ishtar Struct** (`src/ishtar/mod.rs`) - Main application state containing:
   - Widget manager for UI components
   - Current mode (Command/Modify/Selection)
   - Cursor position and state
   - Logger and clipboard

2. **Widget System** - Based on the `IshtarSelectable` trait:
   - Each UI component implements this trait
   - WidgetManager handles widget lifecycle and focus
   - Components include:
     - WriteableArea - Text editing panes
     - CommandInterpreter - Command line interface
     - KeybindHandler - Keyboard shortcuts
     - FileManager - File browsing

3. **Event Handling** - Key events flow through:
   - Main event loop in `Ishtar::run()`
   - Key binding detection and processing
   - Mode-specific key handling
   - Widget-specific key processing

4. **Rendering** - Uses Ratatui for terminal rendering:
   - Each widget implements a `renderize` method
   - Frame buffer updated each frame
   - Cursor positioning based on active widget

## Refactorings Needed

1. **Externalize the `isht` dependency** - The `isht` crate appears to be missing or not properly referenced. Need to either:
   - Find the actual `isht` crate and add it as a proper dependency
   - Inline the necessary configuration structures
   - Remove the dependency if it's not actually used

2. **Improve error handling** - Several places use `unwrap()` and `panic!()` which should be replaced with proper error propagation:
   - In `Ishtar::get_configs()`
   - In various widget manager methods
   - Throughout the codebase where `unwrap()` is used

3. **Widget manager improvements**:
   - Replace `panic!()` in widget lookup with proper error handling
    - Add widget registration/deregistration capabilities

4. **Code organization**:
   - Split large files like `src/ishtar/mod.rs` into smaller, focused modules
   - Group related functionality (e.g., all cursor-related methods)
   - Improve module documentation

5. **Performance optimizations**:
   - Review gap buffer usage in text areas
   - Consider lazy rendering for large files
   - Optimize widget layout calculations

6. **Testing**:
   - Add unit tests for core functionality
   - Add integration tests for key workflows
   - Consider property-based testing for text manipulation

7. **Feature completeness**:
   - Complete syntax highlighting for more languages
   - Add search and replace functionality
   - Improve file manager capabilities
   - Add more configuration options

## License

This project is licensed under the MIT License - see the LICENSE file for details.