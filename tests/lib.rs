#![feature(plugin,const_fn)]
#![plugin(stainless)]

extern crate directory_scanner;

#[cfg(test)]
mod tests {
    pub use directory_scanner::DirectoryScanner;
    pub use directory_scanner::ResultFormat;
    pub use std::path::PathBuf;
    pub use std::sync::mpsc::channel;
    pub use std::sync::{Arc, Mutex};

    describe! stainless {

        before_each {
            let mut path = PathBuf::from("./tests/fixtures/dir-with-11-files/");
        }

        describe! flat_format {

            before_each {
                let result_format = ResultFormat::Flat;
            }

            it "returns finds all the files" {
                let mut directory_scanner = DirectoryScanner::new(path, result_format);
                let results = directory_scanner.scan();
                assert_eq!(results.len(), 11);
            }

            it "returns the results as a list of strings" {
                let mut directory_scanner = DirectoryScanner::new(path, result_format);
                let results = directory_scanner.scan();
                assert_eq!(results[0], "./tests/fixtures/dir-with-11-files/file-01");
                assert_eq!(results.last().unwrap(), "./tests/fixtures/dir-with-11-files/file-11");
            }

            describe! with_sub_directories {

                before_each {
                    let result_format = ResultFormat::Flat;
                    path = PathBuf::from("./tests/fixtures/dir-with-9-files-in-sub-dirs/");
                }

                it "includes the files in the sub dirs" {
                    let mut directory_scanner = DirectoryScanner::new(path, result_format);
                    let results = directory_scanner.scan();
                    assert_eq!(results.len(), 9);
                }

                describe! when_given_a_subscriber {

                    before_each {
                        let result_format = ResultFormat::Flat;
                        path = PathBuf::from("./tests/fixtures/dir-with-5-sub-dirs/");
                        let mut directory_scanner = DirectoryScanner::new(path, result_format);
                        let (transmitter, receiver) = channel();
                        directory_scanner.add_subscriber(transmitter);
                    }

                    it "updates the subscriber after each successful directory scan" {
                        let mut number_of_updates = 0;
                         directory_scanner.scan();
                        for _ in receiver.iter() {
                            number_of_updates = number_of_updates + 1;
                            if number_of_updates == 6 {
                                break;
                            }
                        }
                        assert_eq!(number_of_updates, 6);
                    }
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
