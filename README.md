# send-to-node

Command line tool to send [p2panda](https://github.com/p2panda/handbook)
operations to a node.

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

Create a `.json` file describing the operation you want to send and pipe it
into `send-to-node`. It will read the file from stdin, sign and encode the data
and publish the entry on a node.

The private key is inside of `key.txt`, the program will read it from there. If
the file does not exist yet, a new key will be generated and placed in this
file.

```
USAGE:
    send-to-node [OPTIONS]

OPTIONS:
    -e, --endpoint <ENDPOINT>          Node endpoint URL [default: http://localhost:2020/graphql]
    -f, --file <FILE>                  Optional path to JSON file to parse the operation else reads
                                       piped JSON file from stdin
    -h, --help                         Print help information
    -k, --private-key <PRIVATE_KEY>    Path to private key file [default: key.txt]
    -V, --version                      Print version information
```

## Example

```bash
# Publish an operation
cat schema-definition.json | cargo run
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

## License

[`MIT`](LICENSE)

## Supported by

<img src="https://raw.githubusercontent.com/p2panda/.github/main/assets/ngi-logo.png" width="auto" height="80px"><br />
<img src="https://raw.githubusercontent.com/p2panda/.github/main/assets/eu-flag-logo.png" width="auto" height="80px">

*This project has received funding from the European Union’s Horizon 2020
research and innovation programme within the framework of the NGI-POINTER
Project funded under grant agreement No 871528*
