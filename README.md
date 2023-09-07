# send-to-node

Command line tools to send [p2panda](https://p2panda.org) operations and upload
files to a node.

These tools are simple low-level clients to interact with
[nodes](https://github.com/p2panda/aquadoggo) and help with experimentation and
holding workshops with p2panda and supporting debugging and development.

## Installation

### Pre-compiled binaries

Check out our [Releases](https://github.com/p2panda/send-to-node/releases)
section where we publish binaries for Linux, RaspberryPi, MacOS and Windows.

### Compile it yourself

For the following steps you need a
[Rust](https://www.rust-lang.org/learn/get-started) development environment on
your machine.

```bash
# Download source code
git clone https://github.com/p2panda/send-to-node.git
cd send-to-node

# Compile both binaries
cargo build --release --bins

# Copy binaries into your path (example)
cp ./target/release/send-to-node ~/.local/bin
cp ./target/release/send-blob-to-node ~/.local/bin
```

## Usage

This repository consists of two separate programs `send-to-node` and
`send-blob-to-node`.

The private key used to sign the data you are creating is inside of
`key.txt`, the program will read it from there. If the file does not exist
yet, a new key will be generated and placed in this file.

### Send operations with `send-to-node`

Create a `.json` file describing the operation you want to send and pipe it
into `send-to-node`. It will read the file from stdin, sign and encode the data
and publish the entry on a node. Alternatively you can also use the `[PATH]`
argument to refer to the file.

#### Arguments

```
Usage: send-to-node [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to file containing operation encoded in JSON format

Options:
  -e, --endpoint <ENDPOINT>        Node endpoint URL [default: http://localhost:2020/graphql]
  -k, --private-key <PRIVATE_KEY>  Path to private key file [default: key.txt]
  -h, --help                       Print help
  -V, --version                    Print version
```

#### Example

```bash
# Publish an operation
cat schema-definition.json | send-to-node

# Same, but using the "path" argument
send-to-node schema-definition.json
```

### Upload files with `send-blob-to-node`

Upload any file to a node using `send-blob-to-node`. It will automatically
split the file into blob pieces according to the p2panda specification,
determine it's mime type by looking at the file extension, encode and sign the
operations and send them to the node.

#### Arguments

```
Usage: send-blob-to-node [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to (binary) file which should be uploaded to node

Options:
  -e, --endpoint <ENDPOINT>        Node endpoint URL [default: http://localhost:2020/graphql]
  -k, --private-key <PRIVATE_KEY>  Path to private key file [default: key.txt]
  -h, --help                       Print help
  -V, --version                    Print version
```

#### Example

```bash
# Upload a large file
send-blob-to-node my-large-file.wav
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

## Development

```bash
# Run `send-to-node`
cargo run -- --endpoint http://localhost:2020/graphql

# Run `send-blob-to-node`
cargo run --bin send-blob-to-node -- my-large-file.wav
```

## License

[`MIT`](LICENSE)

## Supported by

<img src="https://raw.githubusercontent.com/p2panda/.github/main/assets/ngi-logo.png" width="auto" height="80px"><br />
<img src="https://raw.githubusercontent.com/p2panda/.github/main/assets/nlnet-logo.svg" width="auto" height="80px"><br />
<img src="https://raw.githubusercontent.com/p2panda/.github/main/assets/eu-flag-logo.png" width="auto" height="80px">

*This project has received funding from the European Union’s Horizon 2020
research and innovation programme within the framework of the NGI-POINTER
Project funded under grant agreement No 871528 and NGI-ASSURE No 957073*
