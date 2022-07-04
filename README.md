# send-to-node

Command line tool to send p2panda operations to a node.

## Usage

```
USAGE:
    send-to-node [OPTIONS]

OPTIONS:
    -d, --document-id <DOCUMENT_ID>
    -e, --endpoint <ENDPOINT>          [default: http://localhost:2020/graphql]
    -h, --help                         Print help information
    -k, --private-key <PRIVATE_KEY>    [default: key.txt]
    -V, --version                      Print version information
```

## Example

```bash
# CREATE operation
cat schema-definition-2.toml | cargo run

# UPDATE operation
cat schema-definition-2.toml | cargo run -- -d <document_id>
```
