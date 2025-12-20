# Batch Invoice Demo

A Rust application for signing and submitting batch payment requests for Bitcoin/RGB invoices. This tool provides functionality to sign batch pay requests using extended private keys (XPRV) and includes a utility to derive XPRV keys from BIP39 mnemonic phrases.

## Features

- **Batch Pay Request Signing**: Sign batch payment requests using extended private keys (XPRV)
- **Mnemonic to XPRV Conversion**: Utility tool to derive XPRV keys from BIP39 mnemonic phrases
- **Secure Key Management**: Uses `secrecy` crate for secure handling of private keys
- **Multiple Network Support**: Supports mainnet, testnet, and regtest networks

## Prerequisites

- Rust (latest stable version)
- Cargo package manager
- Access to the `bitlight-core-derive` crate (located at `../bitlight-core-derive`)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd batch_invoice_demo
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

The main application requires environment variables to be set. You can create a `.env` file in the project root with the following variables:

### Required Environment Variables

- `PUB_DESCRIPTOR`: The public descriptor containing the derivation path
  - Format: `[fingerprint/path]xpub/...`
  - Example: `[9c734ef0/86h/827167h/0h]tpubDDpgp3MoRydYFXayCRRhkj7JUxnuECoVgHx61Po2jNYNhzk9xLyusYGmaH9S4shnQFH8EEDv8PkTcA75186TNEAXWUkAcrZY8ENmkjGe5En/<0;1>/*`

- `XPRV`: Extended private key (xprv/zprv/tprv/uprv format)
  - **Important**: Must be a private key, not a public key (xpub/zpub/tpub/upub)

- `API_KEY`: API key for authentication with the batch pay endpoint

- `INVOICE`: The invoice string to be paid

### Optional Environment Variables

- `FEE_RATE`: Transaction fee rate (default: 5)

### Example `.env` File

```env
PUB_DESCRIPTOR=[9c734ef0/86h/827167h/0h]tpubDDpgp3MoRydYFXayCRRhkj7JUxnuECoVgHx61Po2jNYNhzk9xLyusYGmaH9S4shnQFH8EEDv8PkTcA75186TNEAXWUkAcrZY8ENmkjGe5En/<0;1>/*
XPRV=XPRV
API_KEY=your_api_key_here
INVOICE=your_invoice_string_here
FEE_RATE=5
```

## Usage

### Main Application: Batch Invoice Signing

The main application signs a batch pay request and outputs the signed request in JSON format along with an example curl command.

```bash
cargo run
```

Or using the release binary:

```bash
cargo run --release
```

The application will:
1. Load environment variables from `.env` file
2. Extract the derivation path from `PUB_DESCRIPTOR`
3. Derive the child private key from `XPRV` using the extracted path
4. Sign the batch pay request
5. Output:
   - The signature
   - The complete signed request in JSON format
   - An example curl command to submit the request

#### Example Output

```
Message to sign: fee_rate=5&invoice=your_invoice&timestamp=1234567890
Signature: abc123def456...

=== Signed Batch Pay Request (JSON) ===
{
  "coinselect": null,
  "fee_rate": 5,
  "invoice": "your_invoice_string",
  "signature": "abc123def456...",
  "timestamp": 1234567890
}

=== Example curl command ===
curl -X POST 'https://xxxxx.com/batch_pay' \
  --header 'x-api-key: your_api_key' \
  --header 'authorization: Bearer base64_encoded_pub_descriptor' \
  --header 'Content-Type: application/json' \
  --data-raw '{...}'
```

### Utility: Mnemonic to XPRV

Convert a BIP39 mnemonic phrase to an extended private key (XPRV).

#### Basic Usage

Derive root/master XPRV from mnemonic:

```bash
cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 word3 ... word12"
```

#### With Network Specification

```bash
cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ... word12" --network regtest
```

Supported networks: `mainnet`, `testnet`, `regtest` (default: `regtest`)

#### With Derivation Path

Derive a child XPRV at a specific path:

```bash
cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ... word12" --network regtest --path "m/86h/827167h/0h"
```

#### With Passphrase

Use a BIP39 passphrase:

```bash
cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ... word12" --passphrase "your_passphrase"
```

#### Complete Example

```bash
cargo run --bin mnemonic-to-xprv -- \
  --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --network regtest \
  --path "m/86h/827167h/0h" \
  --passphrase ""
```

## How It Works

### Signing Process

1. **Message Generation**: Creates a canonical message string from the request parameters using the `SignMessage` trait
2. **Hashing**: Hashes the message with SHA256
3. **Key Derivation**: Extracts the derivation path from `PUB_DESCRIPTOR` and derives the child private key from `XPRV`
4. **Signing**: Signs the hash using secp256k1 ECDSA
5. **Encoding**: Encodes the signature as a hexadecimal string

### Descriptor Format

The `PUB_DESCRIPTOR` must follow the format:
```
[fingerprint/path]xpub/...
```

The derivation path is extracted from the part in brackets. For example:
- `[9c734ef0/86h/827167h/0h]tpub...` extracts path `m/86h/827167h/0h`

## Security Considerations

- **Private Keys**: Never commit private keys or `.env` files to version control
- **XPRV Validation**: The application validates that `XPRV` is actually a private key (not a public key)
- **Secure Storage**: Uses the `secrecy` crate to prevent accidental exposure of private keys in logs or memory dumps
- **Environment Variables**: Store sensitive information in `.env` files and ensure they are in `.gitignore`

## Error Handling

The application provides detailed error messages for common issues:

- Missing environment variables
- Invalid XPRV format (e.g., public key provided instead of private key)
- Invalid descriptor format
- Invalid derivation path
- Network parsing errors

## Dependencies

Key dependencies include:
- `bitcoin`: Bitcoin library for key derivation and signing
- `bitlight-core-derive`: Custom derive macro for signing
- `serde`: Serialization/deserialization
- `chrono`: Timestamp generation
- `secrecy`: Secure handling of sensitive data
- `dotenvy`: Environment variable management
- `clap`: Command-line argument parsing

## License

[Specify your license here]

## Contributing

[Add contribution guidelines if applicable]
