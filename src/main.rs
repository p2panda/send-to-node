use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use gql_client::Client;
use p2panda_rs::cddl::{self, validate_cbor};
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

    // Read private key from file
    let mut private_key = String::new();
    let mut file = File::open(args.private_key).expect("Could not open private key file");
    file.read_to_string(&mut private_key)
        .expect("Could not read private key from file");

    // Parse key pair
    let key_pair =
        KeyPair::from_private_key_str(&private_key.replace("\n", "")).expect("Invalid private key");
    let public_key: Author = key_pair.public_key().to_owned().try_into().unwrap();

    // Parse document id
    let document_id = args
        .document_id
        .map(|id| DocumentId::from_str(&id).expect("Invalid document id"));

    // Print some information
    println!("Author: {}", public_key.as_str());

    // Do the requests
    let client = Client::new(args.endpoint);

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

    // Parse operation
    let mut buffer = String::new();
    let stdin = io::stdin();

    for line in BufReader::new(stdin).lines() {
        let value = line.as_ref().unwrap();
        writeln!(buffer, "{}", value).unwrap();
    }
    let operation: Operation = toml::from_str(&buffer).unwrap();
    operation.validate().expect("Invalid operation");

    // Encode operation
    let encoded_operation =
        OperationEncoded::try_from(&operation).expect("Could not encode operation");

    // Validate schema!
    let cddl = match operation.schema() {
        SchemaId::Application(_, _) => None,
        SchemaId::SchemaDefinition(_) => Some(cddl::SCHEMA_V1_FORMAT.as_str()),
        SchemaId::SchemaFieldDefinition(_) => Some(cddl::SCHEMA_FIELD_V1_FORMAT.as_str()),
    };

    /* match cddl {
        Some(str) => {
            validate_cbor(str, &encoded_operation.to_bytes()).expect(
                format!("Operation does not match schema \"{}\"", operation.schema()).as_str(),
            );
        }
        None => (), // Could not validate application schema yet
    } */

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

    let _: PublishEntryResponse = client
        .query_unwrap(&query)
        .await
        .expect("GraphQL mutation `publishEntry` failed");

    println!("Woho!");
}
