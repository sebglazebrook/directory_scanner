use std::path::PathBuf;
use std::fs;


pub enum ResultFormat {
    Flat,
    Nested,
}

pub struct DirectoryScanner {
    root_dir: PathBuf,
}

impl DirectoryScanner {

    pub fn new(root_dir: PathBuf, result_format: ResultFormat) -> DirectoryScanner {
        DirectoryScanner { root_dir: root_dir }
    }

    pub fn scan(&mut self) -> Vec<String> {
        let mut filepaths = vec![];
        match fs::read_dir(&self.root_dir) {
            Ok(read_dir) => {
                for entry in read_dir {
                    match entry {
                        Ok(entry) => {
                            let filetype = entry.file_type().unwrap();
                            if filetype.is_file() {
                                filepaths.push(entry.path().to_str().unwrap().to_string());
                            } else if filetype.is_dir() && !filetype.is_symlink() {
                                let path = PathBuf::from(entry.path().to_str().unwrap().to_string());
                                let sub_filepaths = DirectoryScanner::new(path, ResultFormat::Flat).scan();
                                filepaths.extend(sub_filepaths.clone());
                            }
                        }
                        Err(_) => {  }
                    }
                }
            }
            Err(_) => { } // this should never happen what do we do just in case?
        }
        filepaths
    }
}
