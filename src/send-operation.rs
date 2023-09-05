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
    // 1. Handle command line arguments set by the user and read piped JSON file from stdin stream
    let args = Args::parse();
    let json = read_stdin();

    // 2. Prepare GraphQL client making request against our p2panda node
    let client = Client::new(args.endpoint);

    // 3. Load private key from given file, generate a new one if it doesn't exist yet
    let key_pair = get_key_pair(&args.private_key);
    let public_key = key_pair.public_key();
    println!("▶ Public Key: \"{}\"", public_key);

    // 4. Parse operation from JSON file or stdin, it comes as a JSON string
    let operation: PlainOperation = serde_json::from_str(&json).unwrap();

    // 5. Send `nextArgs` GraphQL query to get the arguments from the node to create the next entry
    let query = format!(
        r#"
            {{
                nextArgs(publicKey: "{}", viewId: {}) {{
                    logId
                    seqNum
                    skiplink
                    backlink
                }}
            }}
        "#,
        public_key,
        // Set `viewId` when `previous` is given in operation
        operation
            .previous()
            .map_or("null".to_owned(), |id| format!("\"{}\"", id)),
    );

    let response: NextArgsResponse = client
        .query_unwrap(&query)
        .await
        .expect("GraphQL query to fetch `nextArgs` failed");
    let args = response.next_args;

    // 6. Create p2panda data! Encode operation, sign and encode entry
    let encoded_operation = encode_plain_operation(&operation).expect("Could not encode operation");
    let encoded_entry = sign_and_encode_entry(
        &args.log_id,
        &args.seq_num,
        args.skiplink.as_ref(),
        args.backlink.as_ref(),
        &encoded_operation,
        &key_pair,
    )
    .expect("Could not sign and encode entry");
    println!("▶ Operation Id: \"{}\"", encoded_entry.hash());

    // 7. Publish operation and entry with GraphQL `publish` mutation
    let query = format!(
        r#"
            mutation Publish {{
                publish(entry: "{}", operation: "{}") {{
                    logId
                    seqNum
                    skiplink
                    backlink
                }}
            }}
        "#,
        encoded_entry, encoded_operation
    );

    client
        .query_unwrap::<PublishResponse>(&query)
        .await
        .expect("GraphQL mutation `publish` failed");

    println!("\nWoho! ヽ(￣(ｴ)￣)ﾉ");
}

/// Helper method to read string from stdin.
fn read_stdin() -> String {
    let mut buffer = String::new();
    let stdin = io::stdin();

    for line in BufReader::new(stdin).lines() {
        let value = line.as_ref().unwrap();
        writeln!(buffer, "{}", value).unwrap();
    }

    buffer
}

/// Helper method to read a file.
fn read_file(path: &PathBuf) -> String {
    let mut content = String::new();
    let mut file = File::open(path).unwrap_or_else(|_| panic!("Could not open file {:?}", path));
    file.read_to_string(&mut content)
        .unwrap_or_else(|_| panic!("Could not read from file {:?}", path));
    content
}

/// Helper method to write a file.
fn write_file(path: &PathBuf, content: &str) {
    let mut file =
        File::create(path).unwrap_or_else(|_| panic!("Could not create file {:?}", path));
    write!(&mut file, "{}", content).unwrap();
}

/// Helper method to read a private key from a file, deriving a key pair from it. If it doesn't
/// exist yet, a new key pair will be generated automatically.
fn get_key_pair(path: &PathBuf) -> KeyPair {
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
