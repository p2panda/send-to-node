// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Context, Result};
use gql_client::Client as GraphQLClient;
use p2panda_rs::document::DocumentViewId;
use p2panda_rs::entry::encode::sign_and_encode_entry;
use p2panda_rs::entry::traits::AsEncodedEntry;
use p2panda_rs::entry::{LogId, SeqNum};
use p2panda_rs::hash::Hash;
use p2panda_rs::identity::{KeyPair, PublicKey};
use p2panda_rs::operation::encode::encode_plain_operation;
use p2panda_rs::operation::plain::PlainOperation;
use p2panda_rs::operation::traits::Actionable;
use p2panda_rs::operation::OperationId;
use serde::Deserialize;

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

pub struct Client {
    client: GraphQLClient,
    next_log_id: Option<LogId>,
}

impl Client {
    pub fn new(endpoint: &str) -> Self {
        Self {
            client: GraphQLClient::new(endpoint),
            next_log_id: None,
        }
    }

    pub async fn next_args(
        &self,
        public_key: &PublicKey,
        previous: Option<&DocumentViewId>,
    ) -> Result<NextArguments> {
        let is_update_or_delete = previous.is_some();

        // Send `nextArgs` GraphQL query to get the arguments from the node to create the next
        // entry
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
            previous.map_or("null".to_owned(), |id| format!("\"{}\"", id)),
        );

        let args = match (is_update_or_delete, self.next_log_id) {
            // If this is an update or delete operation, or the first call for this client (we
            // haven's cached next log id yet) then query the node for next args.
            (true, _) | (false, None) => {
                let response: NextArgsResponse = self
                    .client
                    .query_unwrap(&query)
                    .await
                    .map_err(|err| anyhow!(err))
                    .context("GraphQL query to fetch `nextArgs` failed")?;
                response.next_args
            }
            // If this is a create operation and we already have next log id cached then compose
            // the next args without making a call to the node.
            (false, Some(next_log_id)) => {
                NextArguments {
                    seq_num: SeqNum::default(),
                    log_id: next_log_id,
                    backlink: None,
                    skiplink: None,
                }
            }
        };

        Ok(args)
    }

    pub async fn sign_and_send(
        &mut self,
        key_pair: &KeyPair,
        operation: &PlainOperation,
    ) -> Result<OperationId> {
        let public_key = key_pair.public_key();
        let is_create = operation.previous().is_none();

        let args = self.next_args(&public_key, operation.previous()).await?;

        // Create p2panda data! Encode operation, sign and encode entry
        let encoded_operation =
            encode_plain_operation(&operation).context("Could not encode operation")?;
        let encoded_entry = sign_and_encode_entry(
            &args.log_id,
            &args.seq_num,
            args.skiplink.as_ref(),
            args.backlink.as_ref(),
            &encoded_operation,
            &key_pair,
        )
        .context("Could not sign and encode entry")?;

        // Publish operation and entry with GraphQL `publish` mutation
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

        let response = self
            .client
            .query_unwrap::<PublishResponse>(&query)
            .await
            .map_err(|err| anyhow!(err))
            .context("GraphQL mutation `publish` failed")?;

        // If we created a new document then we should increment the clients cached next_log_id.
        if is_create {
            let mut current_log_id = response.publish.log_id;
            self.next_log_id = Some(current_log_id.next().expect("There is a next log id"));
        }

        Ok(encoded_entry.hash().into())
    }
}
