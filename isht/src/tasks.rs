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
    ReqSearchFile, //Open file manager with this request
    ReqCreateFile,
    ReqRenameFile,
    ReqDeleteFile,
    ReqModifyFile,
    ReqSaveFile,
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
//Some are not here because theyre used in the parser
impl CmdTask {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, String> {
        let value = value.into();
        Ok(match value.as_ref() {
            "SaveMode" => Self::SaveMode,
            "ReturnSavedMode" => Self::ReturnSavedMode,

            "SavePos" => Self::SavePos,
            "MoveSaved" => Self::MoveSaved,
            "CreateWindow" => Self::CreateWindow,
            "DeleteWindow" => Self::DeleteWindow,
            "SetWindowUp" => Self::SetWindowUp,
            "SetWindowDown" => Self::SetWindowDown,
            "CopyToSys" => Self::CopyToSys,
            "CopyToEditor" => Self::CopyToEditor,
            "PasteSys" => Self::PasteSys,
            "PasteEditor" => Self::PasteEditor,
            "SaveFile" => Self::SaveFile,
            "WriteOnFile" => Self::WriteOnFile,
            "CopyFile" => Self::CopyFile,
            //Cmd mode
            "EnterCmd" | "EnterNormal" => Self::EnterNormal,
            "EnterModify" => Self::EnterModify,
            "EnterSelection" => Self::EnterSelection, //Mode to select
            //Selection
            "SelectLine" => Self::SelectLine,
            "DeleteLine" => Self::DeleteLine,
            "CopySelection" => Self::CopySelection,
            "DeleteSelection" => Self::DeleteSelection,

            "RequestSearchFile" => Self::ReqSearchFile, //Open file manager with this request
            "RequestCreateFile" => Self::ReqCreateFile,
            "RequestRenameFile" => Self::ReqRenameFile,
            "RequestDeleteFile" => Self::ReqDeleteFile,

            //Move
            "MoveIOW" => Self::MoveIOW, //Init of word
            "MoveEOW" => Self::MoveEOW, //End of word
            "MoveEOL" => Self::MoveEOL, //End of line
            "MoveIOL" => Self::MoveIOL, //Init of line
            "MoveEOB" => Self::MoveEOB, //End of buffer
            "MoveIOB" => Self::MoveIOB, //Init of buffer

            "Swap" => Self::Swap, //Swap file buffers
            "Null" => Self::Null,
            _ => return Err("Not known task named {value}".to_string()),
        })
    }
}
