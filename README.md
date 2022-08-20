# send-to-node

Command line tool to send [p2panda](https://github.com/p2panda/handbook)
operations to a node.

## Usage

Create a `.toml` file describing the operation you want to send and pipe it
into `send-to-node`. It will read the file from stdin, sign and encode the data
and publish the entry on a node.

The private key is inside of `key.txt`, the program will read it from there. If
the file does not exist yet, a new key will be generated and placed in this
file.

```
USAGE:
    send-to-node [OPTIONS]

OPTIONS:
    -e, --endpoint <ENDPOINT>          [default: http://localhost:2020/graphql]
    -k, --private-key <PRIVATE_KEY>    [default: key.txt]
    -V, --version                      Print version information
    -h, --help                         Print help information
```

## Example

```bash
# publish an operation
cat schema-definition.toml | cargo run
```

## Demo

If you want to reproduce the example operations, you should use the
`key-demo.txt` private key. Make sure there is no data of this author yet on
your node.

Run the whole demo manually step by step or just use that script to run it
automatically:

```bash
./demo.sh
```
