use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write as WriteIo};
use std::path::{Path, PathBuf};

use clap::Parser;
use gql_client::Client;
use p2panda_rs::entry::encode::{encode_entry, sign_entry};
use p2panda_rs::entry::traits::AsEncodedEntry;
use p2panda_rs::hash::Hash;
use p2panda_rs::identity::{Author, KeyPair};
use p2panda_rs::operation::encode::encode_plain_operation;
use p2panda_rs::operation::plain::PlainOperation;
use p2panda_rs::operation::traits::Actionable;
use serde::Deserialize;

/// Send p2panda operations to a node from .toml files via stdin
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(
        short = 'e',
        long,
        value_parser,
        default_value = "http://localhost:2020/graphql"
    )]
    endpoint: String,

    #[clap(short = 'k', long, value_parser, default_value = "key.txt")]
    private_key: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NextEntryArgsResponse {
    next_entry_args: NextEntryArguments,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct PublishEntryResponse {
    publish_entry: NextEntryArguments,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NextEntryArguments {
    log_id: String,
    seq_num: String,
    skiplink: Option<Hash>,
    backlink: Option<Hash>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let stdin = read_stdin();
    let client = Client::new(args.endpoint);

    // Parse key pair
    let key_pair = get_key_pair(&args.private_key);
    let public_key = Author::from(key_pair.public_key());
    println!("Author: {}", public_key.as_str());

    // Parse operation from stdin
    let operation: PlainOperation = serde_json::from_str(&stdin).unwrap();

    // Do the requests
    let query = format!(
        r#"
            {{
                nextEntryArgs(publicKey: "{}", documentViewId: {}) {{
                    logId
                    seqNum
                    skiplink
                    backlink
                }}
            }}
            "#,
        public_key.as_str(),
        operation
            .previous_operations()
            .map_or("null".to_owned(), |id| format!("\"{}\"", id)),
    );

    let response: NextEntryArgsResponse = client
        .query_unwrap(&query)
        .await
        .expect("GraphQL query to fetch `nextEntryArgs` failed");
    let next_entry_args = response.next_entry_args;

    let encoded_operation = encode_plain_operation(&operation).expect("Encode operation");
    let entry = sign_entry(
        &next_entry_args.log_id.parse().unwrap(),
        &next_entry_args.seq_num.parse().unwrap(),
        next_entry_args.skiplink.as_ref(),
        next_entry_args.backlink.as_ref(),
        &encoded_operation,
        &key_pair,
    )
    .expect("Sign entry");

    let encoded_entry = encode_entry(&entry).expect("Encode entry");

    println!("{}", encoded_entry.hash());

    let query = format!(
        r#"
            mutation publishEntry {{
                publishEntry(entry: "{}", operation: "{}") {{
                    logId
                    seqNum
                    skiplink
                    backlink
                }}
            }}
            "#,
        encoded_entry, encoded_operation
    );

    let _response: PublishEntryResponse = client
        .query_unwrap(&query)
        .await
        .expect("GraphQL mutation `publishEntry` failed");

    println!("Woho!");
}

fn read_stdin() -> String {
    let mut buffer = String::new();
    let stdin = io::stdin();

    for line in BufReader::new(stdin).lines() {
        let value = line.as_ref().unwrap();
        writeln!(buffer, "{}", value).unwrap();
    }

    buffer
}

fn read_file(path: &PathBuf) -> String {
    let mut content = String::new();
    let mut file = File::open(path).expect(&format!("Could not open file {:?}", path));
    file.read_to_string(&mut content)
        .expect(&format!("Could not read from file {:?}", path));
    content
}

fn write_file(path: &PathBuf, content: &str) {
    let mut file = File::create(path).expect(&format!("Could not create file {:?}", path));
    write!(&mut file, "{}", content).unwrap();
}

fn get_key_pair(path: &PathBuf) -> KeyPair {
    // Read private key from file or generate a new one
    let private_key = match Path::exists(&path) {
        true => {
            let key = read_file(&path);
            key.replace("\n", "")
        }
        false => {
            let key = hex::encode(KeyPair::new().private_key().to_bytes());
            write_file(&path, &key);
            key
        }
    };

    // Parse key pair
    KeyPair::from_private_key_str(&private_key).expect("Invalid private key")
}
