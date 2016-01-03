#![feature(plugin,const_fn)]
#![plugin(stainless)]

extern crate directory_scanner;

#[cfg(test)]
mod tests {
    pub use directory_scanner::DirectoryScanner;
    pub use directory_scanner::ResultFormat;
    pub use std::path::PathBuf;

    describe! stainless {

        before_each {
            let mut path = PathBuf::from("./tests/fixtures/dir-with-11-files/");
        }

        describe! flat_format {

            before_each {
                let result_format = ResultFormat::Flat;
                let mut directory_scanner = DirectoryScanner::new(path, result_format);
                let results = directory_scanner.scan();
            }

            it "returns finds all the files" {
                assert_eq!(results.len(), 11);
            }

            it "returns the results as a list of strings" {
                assert_eq!(results[0], "./tests/fixtures/dir-with-11-files/file-01");
                assert_eq!(results.last().unwrap(), "./tests/fixtures/dir-with-11-files/file-11");
            }

            describe! with_sub_directories {

                before_each {
                    let result_format = ResultFormat::Flat;
                    path = PathBuf::from("./tests/fixtures/dir-with-9-files-in-sub-dirs/");
                    let mut directory_scanner = DirectoryScanner::new(path, result_format);
                    let results = directory_scanner.scan();
                }

                it "includes the files in the sub dirs" {
                    assert_eq!(results.len(), 9);
                }
            }
        }

        //describe! nested_format {

        //before_each {
        //let result_format = ResultFormat::Flat;
        //let mut directory_scanner = DirectoryScanner::new(result_format);
        //let results = directory_scanner.scan(path);
        //}

        //it "returns finds all the files" {
        //assert_eq!(len(results), 11);
        //}

        //it "returns the results as a list of strings" {
        //assert_eq!(results[0], "first-file");
        //assert_eq!(results[-1], "last-file");
        //}
        //}

    }
}
