use crate::helpers::Vec2;

/// Tracks the terminal cursor position and a saved snapshot of it.
#[derive(Debug, Default)]
pub struct IshtarCursor {
    cursor: Vec2,
    saved_cursor: Vec2,
}

impl IshtarCursor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the cursor to the given position.
    pub fn set_cursor(&mut self, cursor: Vec2) {
        self.cursor = cursor;
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    /// Returns the last saved cursor position.
    pub fn saved_cursor(&self) -> Vec2 {
        self.saved_cursor
    }

    /// Saves the current cursor position for later restoration.
    pub fn save(&mut self) {
        self.saved_cursor = self.cursor;
    }

    /// Restores the cursor to the last saved position.
    pub fn goto_saved(&mut self) {
        self.cursor = self.saved_cursor;
    }
}
