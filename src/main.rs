extern crate directory_scanner;

use directory_scanner::ScannerBuilder;
use std::sync::mpsc::channel;

fn main() {
    let (transmitter, receiver) = channel();

    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path(".");
    scanner_builder = scanner_builder.update_subscriber(transmitter);

    let mut scanner = scanner_builder.build();
    let results = scanner.scan();

    println!("{:?}", results.len());
}
