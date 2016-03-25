use std::path::PathBuf;
use directory_scanner::File;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub files: Vec<File>,
    pub sub_directories: Vec<Directory>,
}

impl Directory {

    pub fn new(path: PathBuf) -> Self {
        Directory { files: vec![], path: path, sub_directories: vec![] }
    }

    pub fn len(&self) -> usize {
        let total = &self.sub_directories.iter()
                       .fold(self.files.len(), |acc, ref directory| acc + directory.len());
        *total
    }

    pub fn push(&mut self, filepath: String) {
        let file = File::new(filepath.clone(), self.path.clone());
        if !self.files.contains(&file) {
            debug!("Adding file {:?} to dir {:?}", filepath, self.path);
            self.files.push(file);
        }
    }

    pub fn extend(&mut self, other: &Directory) {
        // TODO this will make to make sure the other is not higher up the tree then self right?
        if !self.sub_directories.contains(&other) {
            debug!("Extending dir with {:?}", other.path);
            self.sub_directories.push(other.clone());
            debug!("Directory size = {}", self.len());
        }
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut result = vec![];
        let mut flattened_files = vec![];
        for file in &self.files {
            flattened_files.push(file.as_string());
        }
        result.extend(flattened_files.clone());
        for directory in &self.sub_directories {
            result.extend(directory.flatten());
        }
        result
    }

    // TODO can this returns borrows instead?
    pub fn contents(&self) -> Vec<String> {
        self.flatten()
    }

}

impl PartialEq for Directory {

    fn eq(&self, other: &Self) -> bool {
        self.path.cmp(&other.path) == Ordering::Equal
    }

}
