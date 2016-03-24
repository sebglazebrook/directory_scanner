use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct File {
    basename: String,
    dirname: PathBuf,
}

impl File {

    pub fn new(basename: String, dirname: PathBuf) -> Self {
        let new_dir;
        if dirname.starts_with("./") {
            new_dir = unshift(dirname);
        } else {
            new_dir = dirname;
        }
        File { basename: basename, dirname: new_dir }
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

pub fn unshift(path: PathBuf) -> PathBuf {
    let string = path.clone().into_os_string().into_string().unwrap();
    match string.find("/") { // only working on unix type systems now
        None => { path.clone() },
        Some(index) => {
            let (_, last) =  string.split_at(index + 1);
            PathBuf::from(last)
        }
    }
}
