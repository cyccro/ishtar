use std::error::Error;

use super::logger::LogLevel;
use crate::{
    ishtar::Ishtar,
    widgets::{CmdTask, IshtarMode},
};

impl Ishtar {
    /// Executes a list of tasks in order.
    pub fn handle_tasks(&mut self, tasks: &[CmdTask]) {
        // Collect to avoid borrow issues when tasks reference self.
        let tasks: Vec<CmdTask> = tasks.to_vec();
        for task in &tasks {
            let _ = self.handle_task(task);
        }
    }

    /// Dispatches a single `CmdTask`.
    ///
    /// `Null` and `Continue` are no-ops. All other variants are handled here or
    /// delegated to the appropriate widget.
    pub fn handle_task(&mut self, task: &CmdTask) -> Result<(), Box<dyn Error + Send + Sync>> {
        if matches!(task, CmdTask::Null | CmdTask::Continue) {
            return Ok(());
        }
        self.display(format!("{task:?}"), LogLevel::Info);
        match task {
            CmdTask::SaveMode => self.mode.save_mode(),
            CmdTask::ReturnSavedMode => self.mode.goto_saved(),

            CmdTask::CopyToSys | CmdTask::CopyToEditor => {
                let Some(data) = self.widgets_manager.writer_mut().get_selection() else {
                    self.handle_task(&CmdTask::EnterModify)?;
                    return Ok(());
                };
                match task {
                    CmdTask::CopyToSys => {
                        self.clipboard.set(data)?;
                    }
                    _ => self.clipboard.set_virtual(data),
                }
                self.handle_task(&CmdTask::EnterModify)?;
            }

            CmdTask::SelectLine => {
                self.widgets_manager.writer_mut().goto_init_of_line();
                self.change_mode(IshtarMode::Selection);
                self.widgets_manager.writer_mut().goto_end_of_line();
            }

            CmdTask::PasteSys | CmdTask::PasteEditor => {
                let content = if matches!(task, CmdTask::PasteEditor) {
                    self.clipboard.get_virtual().clone()
                } else {
                    self.clipboard.get()?
                };
                let data = self.widgets_manager.writer_mut().paste(&content);
                self.handle_task(&data)?;
            }

            CmdTask::DeleteLine => self.widgets_manager.writer_mut().delete_line(),

            CmdTask::SavePos => self.save_position(),
            CmdTask::MoveSaved => self.cursor.goto_saved(),

            CmdTask::Write(content) => {
                self.widgets_manager.writer_mut().paste(content);
            }
            CmdTask::CreateWindow => self.widgets_manager.writer_mut().create_area(),
            CmdTask::DeleteWindow => {
                self.widgets_manager.writer_mut().delete_current_area();
            }
            CmdTask::SetWindowUp => self.widgets_manager.writer_mut().set_focus_back(),
            CmdTask::SetWindowDown => self.widgets_manager.writer_mut().set_focus_next(),

            CmdTask::MoveIOL => self.widgets_manager.writer_mut().goto_init_of_line(),
            CmdTask::MoveEOL => self.widgets_manager.writer_mut().goto_end_of_line(),
            CmdTask::MoveIOB => self.widgets_manager.writer_mut().goto_init_of_file(),
            CmdTask::MoveEOB => self.widgets_manager.writer_mut().goto_end_of_file(),
            CmdTask::MoveToLine(n) => self.widgets_manager.writer_mut().move_y(*n as i16),
            CmdTask::MoveToRow(n) => self.widgets_manager.writer_mut().move_x(*n as i16),

            CmdTask::EnterNormal => self.change_mode(IshtarMode::Cmd),
            CmdTask::EnterModify => self.change_mode(IshtarMode::Modify),
            CmdTask::EnterSelection => self.change_mode(IshtarMode::Selection),

            CmdTask::ModifyFile(f) => self.widgets_manager.writer_mut().open_file(f.into()),

            CmdTask::SaveFile => {
                if self.widgets_manager.writer().file_name().is_some() {
                    self.widgets_manager.writer().save(&self.current_path)?;
                } else {
                    self.widgets_manager
                        .cmd_mut()
                        .request_data("Give the file a name ", CmdTask::ReqSaveFile);
                }
            }
            CmdTask::SaveFileAs(msg) => {
                let writer = self.widgets_manager.writer_mut();
                writer.modify_file_name(msg);
                writer.save(&self.current_path)?;
            }

            CmdTask::Multi(tasks) => {
                for task in tasks {
                    self.handle_task(task)?;
                }
            }

            CmdTask::Log(s) => {
                self.display(s, LogLevel::Info);
            }
            CmdTask::Warn(s) => {
                self.display(s, LogLevel::Warn);
            }

            CmdTask::FocusNext => self.widgets_manager.focus_next(),
            CmdTask::FocusPrevious => self.widgets_manager.focus_previous(),
            CmdTask::FocusWidget(i) => self.widgets_manager.set_focus(*i),
            CmdTask::FocusDirection(side) => self.widgets_manager.focus_direction(*side),
            CmdTask::Reset => {
                self.widgets_manager.writer_mut().reset();
            }
            CmdTask::Exit => self.exit = true,

            // Tasks that are handled by widgets directly or not yet implemented.
            other => {
                self.display(format!("Unhandled task: {other:?}"), LogLevel::Warn);
            }
        }
        Ok(())
    }
}
