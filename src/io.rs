// SPDX-License-Identifier: MIT

use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write as WriteIo};
use std::path::{Path, PathBuf};

use p2panda_rs::identity::KeyPair;

/// Helper method to read string from stdin.
#[allow(dead_code)]
pub fn read_stdin() -> String {
    let mut buffer = String::new();
    let stdin = std::io::stdin();

    for line in BufReader::new(stdin).lines() {
        let value = line.as_ref().unwrap();
        writeln!(buffer, "{}", value).unwrap();
    }

    buffer
}

/// Helper method to read a file.
pub fn read_file(path: &PathBuf) -> String {
    let mut content = String::new();
    let mut file = File::open(path).unwrap_or_else(|_| panic!("Could not open file {:?}", path));
    file.read_to_string(&mut content)
        .unwrap_or_else(|_| panic!("Could not read from file {:?}", path));
    content
}

/// Helper method to write a file.
pub fn write_file(path: &PathBuf, content: &str) {
    let mut file =
        File::create(path).unwrap_or_else(|_| panic!("Could not create file {:?}", path));
    write!(&mut file, "{}", content).unwrap();
}

/// Helper method to read a private key from a file, deriving a key pair from it. If it doesn't
/// exist yet, a new key pair will be generated automatically.
pub fn get_key_pair(path: &PathBuf) -> KeyPair {
    // Read private key from file or generate a new one
    let private_key = if Path::exists(path) {
        let key = read_file(path);
        key.replace('\n', "")
    } else {
        let key = hex::encode(KeyPair::new().private_key().to_bytes());
        write_file(path, &key);
        key
    };

    // Derive key pair from private key
    KeyPair::from_private_key_str(&private_key).expect("Invalid private key")
}
