// SPDX-License-Identifier: MIT

mod client;

use std::path::PathBuf;

use clap::Parser;

use crate::client::Client;

/// Upload any file to a p2panda node.
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

    /// Path to (binary) file which should be uploaded to node.
    #[clap(value_name = "PATH")]
    file: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = Client::new(&args.endpoint);
}
