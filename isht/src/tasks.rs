#[derive(Debug, Clone)]
pub enum CmdTask {
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
    CreateFile,
    RenameFile,
    DeleteFile,
    WriteOnFile,
    SaveFile,
    CopyFile,
    GetFileContent,
    Write(String), //writes on the current buffer; SysClip and EditorClip are reserved words for
    //System and Editor clipboard respectively
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
}
impl CmdTask {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, String> {
        let value = value.into();
        Ok(match value.as_ref() {
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
            "CreateFile" => Self::CreateFile,
            "RenameFile" => Self::RenameFile,
            "SaveFile" => Self::SaveFile,
            "DeleteFile" => Self::DeleteFile,
            "WriteOnFile" => Self::WriteOnFile,
            "CopyFile" => Self::CopyFile,
            "GetFileContent" => Self::GetFileContent,
            //Cmd mode
            "EnterCmd" | "EnterNormal" => Self::EnterNormal,
            "EnterModify" => Self::EnterModify,
            "EnterSelection" => Self::EnterSelection, //Mode to select
            //Selection
            "SelectLine" => Self::SelectLine,
            "DeleteLine" => Self::DeleteLine,
            "CopySelection" => Self::CopySelection,
            "DeleteSelection" => Self::DeleteSelection,
            //Move
            "MoveIOW" => Self::MoveIOW, //Init of word
            "MoveEOW" => Self::MoveEOW, //End of word
            "MoveEOL" => Self::MoveEOL, //End of line
            "MoveIOL" => Self::MoveIOL, //Init of line
            "MoveEOB" => Self::MoveEOB, //End of buffer
            "MoveIOB" => Self::MoveIOB, //Init of buffer

            "Swap" => Self::Swap, //Swap file buffers
            _ => return Err("Not known task named {value}".to_string()),
        })
    }
}
