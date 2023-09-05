// SPDX-License-Identifier: MIT

use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write as WriteIo};
use std::path::{Path, PathBuf};

use clap::Parser;
use gql_client::Client;
use p2panda_rs::entry::encode::sign_and_encode_entry;
use p2panda_rs::entry::traits::AsEncodedEntry;
use p2panda_rs::entry::{LogId, SeqNum};
use p2panda_rs::hash::Hash;
use p2panda_rs::identity::KeyPair;
use p2panda_rs::operation::encode::encode_plain_operation;
use p2panda_rs::operation::plain::PlainOperation;
use p2panda_rs::operation::traits::Actionable;
use serde::Deserialize;

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
    file: Option<PathBuf>,
}

/// GraphQL response for `nextArgs` query.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NextArgsResponse {
    next_args: NextArguments,
}

/// GraphQL response for `publish` mutation.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct PublishResponse {
    publish: NextArguments,
}

/// GraphQL response giving us the next arguments to create an Bamboo entry.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NextArguments {
    log_id: LogId,
    seq_num: SeqNum,
    skiplink: Option<Hash>,
    backlink: Option<Hash>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
