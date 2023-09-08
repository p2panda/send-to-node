// SPDX-License-Identifier: MIT

mod client;
mod io;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use indicatif::{HumanBytes, ProgressBar};
use p2panda_rs::document::DocumentViewId;
use p2panda_rs::operation::plain::PlainOperation;
use p2panda_rs::operation::OperationBuilder;
use p2panda_rs::schema::validate::MAX_BLOB_PIECE_LENGTH;
use p2panda_rs::schema::SchemaId;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

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
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = Client::new(&args.endpoint);

    // Load private key from given file, generate a new one if it doesn't exist yet
    let key_pair = io::get_key_pair(&args.private_key);
    let public_key = key_pair.public_key();
    println!("▶ Public Key: \"{}\"", public_key);

    // Read file in chunks
    let file = File::open(&args.file).await?;
    let metadata = file.metadata().await?;
    let file_size = metadata.len();
    let mime_type = match mime_guess::from_path(&args.file).first() {
        Some(guessed_type) => guessed_type.to_string(),
        None => "application/octet-stream".into(),
    };
    let expected_blob_pieces = file_size / MAX_BLOB_PIECE_LENGTH as u64;
    println!(
        "▶ File: \"{}\" ({}, {} blob pieces)",
        mime_type,
        HumanBytes(file_size),
        expected_blob_pieces
    );

    let mut reader = BufReader::with_capacity(MAX_BLOB_PIECE_LENGTH, file);
    let mut buf: [u8; MAX_BLOB_PIECE_LENGTH] = [0; MAX_BLOB_PIECE_LENGTH];

    let bar = ProgressBar::new(file_size);
    let mut blob_pieces: Vec<DocumentViewId> = Vec::with_capacity(expected_blob_pieces as usize);

    while let Ok(size) = reader.read(&mut buf[..]).await {
        bar.inc(size as u64);

        if size == 0 {
            break;
        }

        // Send blob piece to node
        let operation = OperationBuilder::new(&SchemaId::BlobPiece(1))
            .fields(&[("data", buf[..size].into())])
            .build()?;
        let operation_id = client
            .sign_and_send(&key_pair, &PlainOperation::from(&operation))
            .await?;
        blob_pieces.push(operation_id.into());
    }

    bar.finish();

    let operation = OperationBuilder::new(&SchemaId::Blob(1))
        .fields(&[
            ("pieces", blob_pieces.into()),
            ("length", (file_size as i64).into()),
            ("mime_type", mime_type.into()),
        ])
        .build()?;
    let operation_id = client
        .sign_and_send(&key_pair, &PlainOperation::from(&operation))
        .await?;

    println!("▶ Blob Document Id: \"{}\"", operation_id);
    println!("\nWoho! ヽ(￣(ｴ)￣)ﾉ");

    Ok(())
}
