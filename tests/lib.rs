#![feature(plugin,const_fn)]
#![plugin(stainless)]

extern crate directory_scanner;

#[cfg(test)]
mod tests {
    pub use directory_scanner::ScannerBuilder;
    pub use directory_scanner::DirectoryScanner;
    pub use std::path::PathBuf;
    pub use std::sync::mpsc::channel;
    pub use std::sync::{Arc, Mutex};

    describe! directory_scanner {

        before_each {
            let mut scanner_builder = ScannerBuilder::new();
            scanner_builder = scanner_builder.start_from_path("./tests/fixtures/dir-with-11-files/");
        }

        it "returns finds all the files" {
            let mut scanner = scanner_builder.build();
            let results = scanner.scan();
            assert_eq!(results.len(), 11);
        }

        it "returns the results as a list of strings" {
            let mut scanner = scanner_builder.build();
            let results = scanner.scan();
            assert_eq!(results.flatten()[0], "tests/fixtures/dir-with-11-files/file-01");
            assert_eq!(results.flatten().last().unwrap(), "tests/fixtures/dir-with-11-files/file-11");
        }

        describe! with_sub_directories {

            before_each {
                scanner_builder = scanner_builder.start_from_path("tests/fixtures/dir-with-9-files-in-sub-dirs/");
            }

            it "includes the files in the sub dirs" {
                scanner_builder = scanner_builder.max_threads(1);
                let mut scanner = scanner_builder.build();
                let results = scanner.scan();
                assert_eq!(results.len(), 9);
            }

            describe! when_given_a_subscriber {

                before_each {
                    let (transmitter, receiver) = channel();
                    scanner_builder = scanner_builder.start_from_path("tests/fixtures/dir-with-5-sub-dirs/");
                    scanner_builder = scanner_builder.update_subscriber(Arc::new(Mutex::new(transmitter.clone())));
                }

                it "updates the subscriber after each successful directory scan" {
                    {
                        let mut scanner = scanner_builder.build();
                        scanner.scan();
                    }
                    let mut number_of_updates = 0;
                    for _ in receiver.iter() {
                        number_of_updates = number_of_updates + 1;
                        if number_of_updates == 6 { break; } // todo remove this hack
                    }
                    assert_eq!(number_of_updates, 6);
                }
            }
        }

        describe! concurrency_limit {

            before_each {
                scanner_builder = scanner_builder.start_from_path("tests/fixtures/dir-with-10-sub-dirs/");
            }

            it "defaults to 9" {
                let mut scanner = scanner_builder.build();
                scanner.scan();
                assert_eq!(scanner.max_concurrency_reached, 9);
            }

            describe! when_given_a_custom_concurrency_limit {

                before_each {
                    scanner_builder = scanner_builder.max_threads(3);
                }

                it "does not exceed the given limit" {
                    let mut scanner = scanner_builder.build();
                    scanner.scan();
                    assert_eq!(scanner.max_concurrency_reached, 2);
                }
            }
        }
    }
}
