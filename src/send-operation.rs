// SPDX-License-Identifier: MIT

mod client;
mod io;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use p2panda_rs::operation::plain::PlainOperation;

use crate::client::Client;

/// Send p2panda operations to a node from .json files via stdin.
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Node endpoint URL.
    #[clap(
        short = 'e',
        long,
        value_parser,
        default_value = "http://localhost:2020/graphql"
    )]
    endpoint: String,

    /// Path to private key file.
    #[clap(short = 'k', long, value_parser, default_value = "key.txt")]
    private_key: PathBuf,

    /// Path to file containing operation encoded in JSON format.
    #[clap(value_name = "PATH")]
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Handle command line arguments set by the user
    let args = Args::parse();

    // Read JSON from stdin stream or from specified file argument
    let json = match args.file {
        Some(path) => io::read_file(&path),
        None => io::read_stdin(),
    };

    // Prepare GraphQL client making request against our p2panda node
    let mut client = Client::new(&args.endpoint);

    // Load private key from given file, generate a new one if it doesn't exist yet
    let key_pair = io::get_key_pair(&args.private_key);
    let public_key = key_pair.public_key();
    println!("▶ Public Key: \"{}\"", public_key);

    // Parse operation from JSON file or stdin, it comes as a JSON string
    let operation: PlainOperation = serde_json::from_str(&json).context("Invalid JSON format")?;

    // Sign operation and send it to node!
    let operation_id = client.sign_and_send(&key_pair, &operation).await?;
    println!("▶ Operation Id: \"{}\"", operation_id);
    println!("\nWoho! ヽ(￣(ｴ)￣)ﾉ");

    Ok(())
}
