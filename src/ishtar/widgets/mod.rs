use ratatui::{crossterm::event::KeyCode, layout::Rect, Frame};

#[derive(Debug, Clone)]
pub enum CmdTask {
    Multi(Vec<CmdTask>),
    SaveMode,
    ReturnSavedMode,
    //position
    SavePos,
    MoveSaved,
    //window
    CreateWindow,
    DeleteWindow,
    SetWindowUp,   //goes to the window above
    SetWindowDown, //goes to the window below
    //clipboard
    CopyToSys,
    CopyToEditor,
    PasteSys,
    PasteEditor,
    //fs
    CreateFile(String),         //File target
    ModifyFile(String),         //File target
    RenameFile(String, String), //By now not avaible in isht
    DeleteFile(String),         //File target
    SaveFileAs(String),
    SaveFile,
    WriteOnFile,
    CopyFile,      //Copies the file path to clipboard
    Write(String), //writes on the current buffer; SysClip and EditorClip are reserved words for
    //System and Editor clipboard respectively
    ReqSearchCurr, //Open file manager with this request on the current directory
    ReqSearchRoot, //Open file manager with this request on the root directory. Actually, will
    //reset the directory to be on the root dir(the one ishtar was opened firstly)
    ReqCreateFile,
    ReqRenameFile,
    ReqDeleteFile,
    ReqModifyFile,
    ReqSaveFile,

    StopSearch,
    //Cmd mode
    EnterNormal,
    EnterModify,
    EnterSelection, //mode to select
    //Cmds
    ExecCmd(String), //executes the following commands, its terminal ones.
    ExecutePrompt(String),
    //Selection
    SelectLine,
    DeleteLine,
    CopySelection,
    DeleteSelection,
    //Move
    MoveToLine(u32),
    MoveToRow(u32),
    MoveIOW, //init of word
    MoveEOW, //end of word
    MoveEOL, //end of line
    MoveIOL, //init of line
    MoveEOB, //end of buffer
    MoveIOB, // init of buffer

    Swap, //swap file buffers

    //Edtior internals
    Null,     //For functions that need returning some task but dont want doing anything
    Continue, //For functions that need continuing the function without doing anything and returning
    SetPriority(u8),
    Exit,
    Reset,
    Log(String),
    Warn(String),
}

pub trait IshtarSelectable {
    fn keydown(&mut self, key: KeyCode) -> CmdTask;
    fn can_render(&self) -> bool {
        true
    }
    fn renderize(&self, f: &mut Frame, area: Rect);
}

pub mod command_interpreter;
pub mod file_manager;
pub mod keybind_handler;
pub mod popup;
pub mod text_area;
pub mod writeable_area;
