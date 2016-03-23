use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    basename: String,
    dirname: PathBuf,
}

impl File {

    pub fn new(basename: String, dirname: PathBuf) -> Self {
        File { basename: basename, dirname: dirname }
    }

    pub fn path(&self) -> PathBuf {
        self.dirname.join(self.basename.clone())
    }

    pub fn file_name(&self) -> PathBuf {
        PathBuf::from(self.basename.clone())
    }

    pub fn as_string(&self) -> String {
        self.path().to_str().unwrap().to_string()
    }

}
