use std::convert::TryFrom;
use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write as WriteIo};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::Parser;
use gql_client::Client;
use p2panda_rs::cddl::{validate_cbor, CddlValidationError};
use p2panda_rs::document::DocumentId;
use p2panda_rs::entry::{sign_and_encode, Entry};
use p2panda_rs::hash::Hash;
use p2panda_rs::identity::{Author, KeyPair};
use p2panda_rs::operation::{AsOperation, Operation, OperationEncoded};
use p2panda_rs::schema::SchemaId;
use p2panda_rs::Validate;
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

    #[clap(short = 'd', long, value_parser)]
    document_id: Option<String>,
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
    let public_key = Author::try_from(key_pair.public_key().clone()).unwrap();
    println!("Author: {}", public_key.as_str());

    // Parse document id
    let document_id = args
        .document_id
        .map(|id| DocumentId::from_str(&id).expect("Invalid document id"));

    // Do the requests
    let query = format!(
        r#"
        {{
            nextEntryArgs(publicKey: "{}", documentId: {}) {{
                logId
                seqNum
                skiplink
                backlink
            }}
        }}
        "#,
        public_key.as_str(),
        document_id.map_or("null".to_owned(), |id| format!("\"{}\"", id.as_str())),
    );

    let response: NextEntryArgsResponse = client
        .query_unwrap(&query)
        .await
        .expect("GraphQL query to fetch `nextEntryArgs` failed");
    let next_entry_args = response.next_entry_args;

    // Parse operation from stdin
    let operation: Operation = toml::from_str(&stdin).unwrap();
    operation.validate().expect("Invalid operation");

    // Encode operation
    let encoded_operation =
        OperationEncoded::try_from(&operation).expect("Could not encode operation");

    // Validate schema!
    validate_schema(&operation.schema(), &encoded_operation.to_bytes()).expect(&format!(
        "Operation does not match schema \"{}\"",
        operation.schema()
    ));

    // Sign and encode entry
    let entry = Entry::new(
        &next_entry_args.log_id.parse().unwrap(),
        Some(&operation),
        next_entry_args.skiplink.as_ref(),
        next_entry_args.backlink.as_ref(),
        &next_entry_args.seq_num.parse().unwrap(),
    )
    .expect("Could not create entry");

    let encoded_entry =
        sign_and_encode(&entry, &key_pair).expect("Could not sign and encode entry");

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
        encoded_entry.as_str(),
        encoded_operation.as_str()
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

fn validate_schema(schema_id: &SchemaId, payload: &[u8]) -> Result<(), CddlValidationError> {
    let cddl_str = match schema_id {
        SchemaId::Application(_, _) => None,
        SchemaId::SchemaDefinition(_) => {
            // @TODO: CDDL definition is invalid for schema_definition_v1 :-)
            // Related issue: https://github.com/p2panda/p2panda/issues/387
            // Some(p2panda_rs::cddl::SCHEMA_V1_FORMAT.as_str()),
            None
        }
        SchemaId::SchemaFieldDefinition(_) => {
            // @TODO: CDDL definition is invalid for schema_field_definition_v1 :-)
            // Related issue: https://github.com/p2panda/p2panda/issues/386
            // Some(p2panda_rs::cddl::SCHEMA_FIELD_V1_FORMAT.as_str()),
            None
        }
    };

    match cddl_str {
        Some(str) => validate_cbor(str, &payload),
        None => Ok(()), // Could not validate application schema yet
    }
}
