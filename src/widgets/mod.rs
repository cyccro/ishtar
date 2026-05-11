use ratatui::{crossterm::event::KeyCode, layout::Rect, Frame};

/// Commands that widgets return to request editor actions.
///
/// Widgets return these from [`IshtarSelectable::keydown`] to signal the
/// editor to perform operations such as mode changes, file I/O, clipboard
/// access, window management, and cursor movement.
#[derive(Debug, Clone)]
pub enum CmdTask {
    // ── Composite ──
    /// Execute multiple tasks in sequence.
    Multi(Vec<CmdTask>),

    // ── Mode management ──
    /// Save the current mode for later restoration.
    SaveMode,
    /// Restore the previously saved mode.
    ReturnSavedMode,

    // ── Cursor position ──
    /// Save the current cursor position.
    SavePos,
    /// Restore the saved cursor position.
    MoveSaved,

    // ── Window / split management ──
    /// Create a new editor split.
    CreateWindow,
    /// Close the current editor split.
    DeleteWindow,
    /// Move focus to the split above.
    SetWindowUp,
    /// Move focus to the split below.
    SetWindowDown,

    // ── Clipboard ──
    /// Copy selection to the system clipboard.
    CopyToSys,
    /// Copy selection to the editor (virtual) clipboard.
    CopyToEditor,
    /// Paste from the system clipboard.
    PasteSys,
    /// Paste from the editor (virtual) clipboard.
    PasteEditor,

    // ── File operations ──
    /// Open a file for editing.
    ModifyFile(String),
    /// Save the current buffer to its associated file.
    SaveFile,
    /// Save the current buffer to a new file path.
    SaveFileAs(String),
    /// Write content to the current buffer at the cursor position.
    Write(String),

    // ── File request prompts ──
    /// Request a filename from the user for `:m` (modify file).
    ReqModifyFile,
    /// Request a filename from the user for `:s` (save new file).
    ReqSaveFile,
    /// Open the file browser at the root directory.
    ReqSearchRoot,
    /// Close the file browser.
    StopSearch,

    // ── Mode transitions ──
    /// Enter command (normal) mode.
    EnterNormal,
    /// Enter insert (modify) mode.
    EnterModify,
    /// Enter selection mode.
    EnterSelection,

    // ── Selection operations ──
    /// Select the current line.
    SelectLine,
    /// Delete the current line.
    DeleteLine,

    // ── Cursor movement ──
    /// Move cursor to the beginning of the line.
    MoveIOL,
    /// Move cursor to the end of the line.
    MoveEOL,
    /// Move cursor to the beginning of the buffer.
    MoveIOB,
    /// Move cursor to the end of the buffer.
    MoveEOB,
    /// Move cursor by `n` lines (positive = down, negative = up).
    MoveToLine(u32),
    /// Move cursor by `n` columns (positive = right, negative = left).
    MoveToRow(u32),

    // ── Editor lifecycle ──
    /// No operation; used as a default return value.
    Null,
    /// Continue processing the current event (not yet consumed).
    Continue,
    /// Exit the editor.
    Exit,
    /// Reset the current buffer (clear content and unset file).
    Reset,

    // ── Logging ──
    /// Log an informational message.
    Log(String),
    /// Log a warning message.
    Warn(String),
}

/// Core trait implemented by every editor widget.
pub trait IshtarSelectable: downcast_rs::DowncastSync {
    /// Handles a key press and returns the resulting task (or `CmdTask::Null`).
    fn keydown(&mut self, key: KeyCode) -> CmdTask;

    /// Returns `false` to skip rendering this widget for the current frame.
    fn can_render(&self) -> bool {
        true
    }

    /// Draws the widget into `f` constrained to `area`.
    fn renderize(&self, f: &mut Frame, area: Rect);
}
downcast_rs::impl_downcast!(sync IshtarSelectable);

pub mod buffer;
pub use buffer::writeable_area::WriteableArea;
pub mod clipboard;
pub use clipboard::*;
pub mod command_interpreter;
pub use command_interpreter::CommandInterpreter;
pub mod cursor;
pub use cursor::*;
pub mod file_manager;
pub mod keybind_handler;
pub mod mode;
pub use mode::*;
pub mod popup;
