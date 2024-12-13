use std::{
    fmt::{Arguments, Display},
    fs::File,
    io::Write,
    ops::{Deref, DerefMut},
    process::Command,
};
#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Fatal,
}

pub struct IshtarLogger {
    f: File,
    queue: String,
}
impl IshtarLogger {
    pub fn new(keep: bool, only_write: bool) -> std::io::Result<Self> {
        if !only_write {
            if cfg!(target_family = "windows") {
                let mut cmd = Command::new("cmd");
                cmd.arg("--Command")
                    .arg("$input | Measure-Object -Line -Word, -Character");
                cmd
            } else {
                let mut cmd = Command::new("gnome-terminal");
                cmd.arg("--")
                    .arg("bash")
                    .arg("-c")
                    .arg("bash -c \"tail -f ./tmp/log.txt; exec tail\"");
                cmd
            }
            .spawn()?;
        }
        Ok(Self {
            f: if keep {
                File::options().append(true).open("./tmp/log.txt")?
            } else {
                File::create("./tmp/log.txt")?
            },
            queue: String::new(),
        })
    }
    fn log_queue(&mut self, level: LogLevel) -> usize {
        let current_time = chrono::prelude::Local::now();
        let content = format!("{level:?}: {}. Sent at {current_time}\n", self.queue);
        self.queue.clear();
        self.f.write(content.as_bytes()).unwrap()
    }
    fn log_data<T: Display>(&mut self, data: T, level: LogLevel) -> usize {
        let current_time = chrono::prelude::Utc::now().naive_local();
        let content = format!("{level:?}: {data}; at {current_time:?}\n");
        self.f.write(content.as_bytes()).unwrap()
    }
    pub fn queue_char(&mut self, c: char) {
        self.queue.push(c);
    }
    pub fn queue<T: ToString>(&mut self, val: &T) {
        self.queue.push_str(&val.to_string());
    }
    pub fn flush(&mut self, level: LogLevel) -> (usize, usize) {
        let len = self.queue.len();
        let bytes = self.log_queue(level);
        (len, bytes)
    }
    pub fn display<T: Display>(&mut self, p: T, level: LogLevel) -> usize {
        self.log_data(p, level)
    }
    pub fn debug(&mut self, p: Arguments<'_>, level: LogLevel) -> usize {
        self.log_data(format!("{:#?}", p), level)
    }
    pub fn buffer(&mut self, buf: &[u8], level: LogLevel) -> usize {
        let buf = String::from_utf8_lossy(buf);
        self.log_data(buf, level)
    }
}
impl Deref for IshtarLogger {
    type Target = File;
    fn deref(&self) -> &Self::Target {
        &self.f
    }
}
impl DerefMut for IshtarLogger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.f
    }
}
