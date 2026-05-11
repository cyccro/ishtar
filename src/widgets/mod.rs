pub mod clipboard;
pub mod file_manager;

use ratatui::{crossterm::event::KeyCode, layout::Rect, Frame};

#[derive(Debug, Clone)]
pub enum CmdTask {
    ///Save the current mode
    SaveMode,
    ///Goes back to the previous saved mode
    ReturnSavedMode,
    ///Saves the current position of the cursor
    SavePos,
    ///Moves the cursor to the saved position
    MoveSaved,
    ///Creates a new window buffer
    CreateWindow,
    ///Deletes the current window buffer
    DeleteWindow,
    ///Goes to the next window buffer
    SetWindowNext,
    ///Goes to the previous window buffer
    SetWindowPrevious,
    ///Yanks the text to the system clipboard
    CopyToSys,
    ///Yanks the text to the editor clipboard
    CopyToEditor,
    ///Paste the text from the system clipboard
    PasteSys,
    ///Paste the text from the editor clipboard
    PasteEditor,
    ///Creates a file on the given path
    CreateFile(String),
    ///Modifies the file on the given path
    ModifyFile(String),
    ///Rename the file on the `target` to be `out`
    RenameFile {
        target: String,
        out: String,
    },
    ///Deletes the file on the given path
    DeleteFile(String),
    ///Saves the content on the current buffer on the file on the given path
    SaveFileAs(String),
    ///Saves the current buffer on the file
    SaveFile,

    WriteOnFile,
    ///Copies the file path to clipboard
    CopyFile,
    ///writes on the current buffer; SysClip and EditorClip are reserved words for System and Editor clipboard respectively
    Write(String),
    ///Open file manager with this request on the current directory
    ReqSearchRoot,

    ReqCreateFile,
    ReqRenameFile,
    ReqDeleteFile,
    ReqModifyFile,
    ReqSaveFile,

    StopSearch,
    ///Enter normal mode
    EnterNormal,
    ///Enter modify mode
    EnterModify,
    ///Enter selection mode
    EnterSelection,
    ///Executes the given command
    ExecCmd(String),
    ///Executes the following terminal commands.
    ExecutePrompt(String),
    ///Selects the current file
    SelectLine,
    ///Deletes the current line
    DeleteLine,
    ///Copy all the selection
    CopySelection,
    ///Deletes all the selection
    DeleteSelection,
    ///Moves to the given line on the buffer
    MoveToLine(u32),
    ///Moves to the given row on the line
    MoveToRow(u32),
    ///Moves to the init of the word
    MoveIOW,
    ///Moves to the end of the word
    MoveEOW,
    ///Moves to the end of the line
    MoveEOL,
    ///Moves to the init of the line
    MoveIOL,
    ///Moves to the end of the buffer
    MoveEOB,
    //Moves to the init of the buffer
    MoveIOB,
    ///Swaps file buffers
    Swap,

    ///Null command, used on functions that need to return some function but dont want to do anything
    Null,
    ///Continue command, used for functions that need to continue without anything    
    Continue,
    ///Exists the editor
    Exit,
    ///Resets the editor
    Reset,
    ///Shows the given content on logs
    Log(String),
    ///Shows the given content on warnings
    Warn(String),
}

pub trait IshtarSelectable {
    fn keydown(&mut self, key: KeyCode) -> CmdTask;
    fn can_render(&self) -> bool {
        true
    }
    fn renderize(&self, f: &mut Frame, area: Rect);
}
