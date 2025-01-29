use copypasta::{ClipboardContext, ClipboardProvider};

pub struct IshtarClipboard {
    service: ClipboardContext,
    virtual_clip: String,
}
impl Default for IshtarClipboard {
    fn default() -> Self {
        Self::new()
    }
}
///A manager for the clipboard, that saves directly into the clipboard or into the vritual
///clipboard. The virtual clipboard is simply a string managed by the application
impl IshtarClipboard {
    pub fn new() -> Self {
        Self {
            service: copypasta::ClipboardContext::new().unwrap(),
            virtual_clip: String::new(),
        }
    }
    ///Gets the content in the clipboard
    pub fn get(&mut self) -> String {
        if let Ok(s) = self.service.get_contents() {
            s
        } else {
            "Error".to_string()
        }
    }
    ///Sets the given content into the clipboard and returns the old one
    pub fn set<S: Into<String>>(&mut self, content: S) -> String {
        let current = self.get();
        self.service.set_contents(content.into()).unwrap();
        current
    }
    ///Appends into the clipboard the given content
    pub fn append<S: Into<String>>(&mut self, content: S) {
        let mut current = self.get();
        current.push_str(&content.into());
        self.service.set_contents(current).unwrap();
    }
    ///Gets the content of the virtual clipboard.
    pub fn get_virtual(&self) -> &String {
        &self.virtual_clip
    }
    ///Sets the given content into the virtual clipboard
    pub fn set_virtual<S: Into<String>>(&mut self, content: S) {
        self.virtual_clip = content.into();
    }
    ///Appends the given content into the clipboard
    pub fn append_virtual<S: Into<String>>(&mut self, content: S) {
        self.virtual_clip.push_str(&content.into());
    }
    ///Swaps the contents of the virtual clipboard and the clipboard
    pub fn swap(&mut self) {
        if let Ok(clip) = self.service.get_contents() {
            self.set(self.virtual_clip.clone());
            self.virtual_clip.clear();
            self.virtual_clip.push_str(&clip);
        } else {
            self.set(self.virtual_clip.clone());
            self.virtual_clip.clear();
        }
    }
}
