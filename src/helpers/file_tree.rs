use std::{
    io::Result,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum FileTree {
    Dir(PathBuf),
    File(PathBuf),
}

impl FileTree {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        let metadata = std::fs::metadata(&path)?; //checking if exists or not, if not, returns error
        Ok(if metadata.is_file() {
            FileTree::File(path)
        } else {
            FileTree::Dir(path)
        })
    }
    pub fn path(&self) -> PathBuf {
        match self {
            Self::Dir(path) => path.clone(),
            Self::File(path) => path.clone(),
        }
    }
    ///Reads the paths inside this returning a vector including them all including folder ones
    pub fn read_paths(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut buf = vec![self.path()];
        if let FileTree::Dir(path) = self {
            let entries = std::fs::read_dir(path)?;
            for entry in entries {
                buf.push(entry?.path());
            }
        }
        Ok(buf)
    }
    ///Read the paths inside all subpaths of this, it will not include folder ones, only file paths
    pub fn read_paths_recursively(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut buf = vec![self.path()];
        if let FileTree::Dir(path) = self {
            internal_recursive(path, &mut buf)?;
        }
        Ok(buf)
    }
}
fn internal_recursive(dir: &Path, target: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let path = entry.path();
        if !metadata.is_dir() {
            target.push(path);
        } else {
            internal_recursive(&path, target)?;
        }
    }
    Ok(())
}
