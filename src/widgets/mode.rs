/// The three editing modes of the editor, inspired by vim's modal editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IshtarMode {
    /// Normal/command mode: navigate and issue commands
    Cmd,
    /// Insert mode: type text into the buffer
    Modify,
    /// Visual mode: select text regions
    Selection,
}

/// Manages the current and previously saved editor mode.
#[derive(Debug, Clone, Copy)]
pub struct IshtarModeManager {
    mode: IshtarMode,
    saved: IshtarMode,
}

impl Default for IshtarModeManager {
    fn default() -> Self {
        Self {
            mode: IshtarMode::Cmd,
            saved: IshtarMode::Cmd,
        }
    }
}

impl IshtarModeManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the active editing mode.
    pub fn current_mode(&self) -> IshtarMode {
        self.mode
    }

    /// Transitions to the given mode.
    pub fn goto_mode(&mut self, mode: IshtarMode) {
        self.mode = mode;
    }

    /// Saves the current mode so it can be restored later.
    pub fn save_mode(&mut self) {
        self.saved = self.mode;
    }

    /// Restores the previously saved mode.
    pub fn goto_saved(&mut self) {
        self.mode = self.saved;
    }
}
