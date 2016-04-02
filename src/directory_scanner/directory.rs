use std::path::PathBuf;
use directory_scanner::File;
use std::cmp::Ordering;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Directory {
    path: PathBuf,
    files: Arc<RwLock<Vec<File>>>,
    sub_directories: Arc<RwLock<Vec<Directory>>>,
}

impl Directory {

    pub fn new(path: PathBuf) -> Self {
        let files = Arc::new(RwLock::new(vec![]));
        let sub_directories = Arc::new(RwLock::new(vec![]));
        Directory { files: files, path: path, sub_directories: sub_directories }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn path_string(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    pub fn len(&self) -> usize {
        let total = &self.sub_directories.read().unwrap().iter()
                       .fold(self.files.read().unwrap().len(), |acc, ref directory| acc + directory.len());
        *total
    }

    pub fn push(&self, filepath: String) {
        let file = File::new(filepath.clone(), self.path.clone());
        if !self.files.read().unwrap().contains(&file) {
            debug!("Adding file {:?} to dir {:?}", filepath, self.path);
            self.files.write().unwrap().push(file);
        }
    }

    pub fn each_sub_directory(&self) -> SubDirectoryIterator {
        SubDirectoryIterator::new(&self.sub_directories)
    }

    pub fn each_file(&self) -> FileIterator {
        FileIterator::new(&self.files)
    }

    pub fn files(&self) -> Vec<File> {
        self.files.read().unwrap().clone()
    }

    pub fn extend(&self, other: &Directory) {
        // TODO this will make to make sure the other is not higher up the tree then self right?
        if !self.sub_directories.read().unwrap().contains(&other) {
            debug!("Extending dir with {:?}", other.path());
            self.sub_directories.write().unwrap().push(other.clone());
            debug!("Directory size = {}", self.len());
        }
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut result = vec![];
        let mut flattened_files = vec![];
        for file in self.each_file() {
            flattened_files.push(file.as_string());
        }
        result.extend(flattened_files.clone());
        for directory in self.each_sub_directory() {
            result.extend(directory.flatten());
        }
        result
    }

    pub fn contents(&self) -> Vec<String> {
        self.flatten()
    }

    pub fn file_contents(&self) -> Vec<File> {
        let mut result = vec![];
        for file in self.each_file() {
            result.push(file.clone());
        }
        for directory in self.each_sub_directory() {
            result.extend(directory.file_contents());
        }
        result
    }

}

impl PartialEq for Directory {

    fn eq(&self, other: &Self) -> bool {
        self.path.cmp(&other.path) == Ordering::Equal
    }

}

pub struct SubDirectoryIterator<'a> {
    sub_directories: &'a Arc<RwLock<Vec<Directory>>>,
    index: usize,
}

impl<'a> SubDirectoryIterator<'a> {

    pub fn new(sub_directories: &'a Arc<RwLock<Vec<Directory>>>) -> Self {
        SubDirectoryIterator { sub_directories: sub_directories, index: 0 }
    }
}

impl<'a> Iterator for SubDirectoryIterator<'a> {
     type Item = Directory;

     fn next(&mut self) -> Option<Directory> {
         match self.sub_directories.read().unwrap().get(self.index) {
             Some(result) => {
                 self.index += 1;
                 Some(result.clone())
             },
             None => None
         }
     }
}

pub struct FileIterator<'a> {
    files: &'a Arc<RwLock<Vec<File>>>,
    index: usize,
}

impl<'a> FileIterator<'a> {

    pub fn new(files: &'a Arc<RwLock<Vec<File>>>) -> Self {
        FileIterator { files: files, index: 0 }
    }
}

impl<'a> Iterator for FileIterator<'a> {
     type Item = File;

     fn next(&mut self) -> Option<File> {
         match self.files.read().unwrap().get(self.index) {
             Some(result) => {
                 self.index += 1;
                 Some(result.clone())
             },
             None => None
         }
     }
}
