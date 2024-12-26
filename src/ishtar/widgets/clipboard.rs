use copypasta::{ClipboardContext, ClipboardProvider};

pub struct IshtarClipboard {
    service: ClipboardContext,
    virtual_clip: String,
}
impl IshtarClipboard {
    pub fn new() -> Self {
        Self {
            service: copypasta::ClipboardContext::new().unwrap(),
            virtual_clip: String::new(),
        }
    }
    pub fn get(&mut self) -> String {
        if let Ok(s) = self.service.get_contents() {
            s
        } else {
            "Error".to_string()
        }
    }
    pub fn set<S: Into<String>>(&mut self, content: S) -> String {
        let current = self.get();
        self.service.set_contents(content.into()).unwrap();
        current
    }
    pub fn append<S: Into<String>>(&mut self, content: S) {
        let mut current = self.get();
        current.push_str(&content.into());
        self.service.set_contents(current).unwrap();
    }
    pub fn get_virtual(&self) -> &String {
        &self.virtual_clip
    }
    pub fn set_virtual<S: Into<String>>(&mut self, content: S) {
        self.virtual_clip = content.into();
    }
    pub fn append_virtual<S: Into<String>>(&mut self, content: S) {
        self.virtual_clip.push_str(&content.into());
    }
    pub fn swap(&mut self) {
        if let Ok(clip) = self.service.get_contents() {
            self.set(self.virtual_clip.clone());
            self.virtual_clip = clip;
        } else {
            self.set(self.virtual_clip.clone());
            self.virtual_clip = "".to_string()
        }
    }
}
