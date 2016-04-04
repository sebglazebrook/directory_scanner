extern crate directory_scanner;

use directory_scanner::ScannerBuilder;

#[allow(unused_variables)]
fn main() {
    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path(".");
    //scanner_builder = scanner_builder.max_threads(2);


    let mut scanner = scanner_builder.build();
    let directory = scanner.scan();


    while !scanner.is_complete() {
    }
    println!("{:?}", directory.len());

}
