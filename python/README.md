# Batch Invoice Demo (Python)

A Python application for signing and submitting batch payment requests for Bitcoin/RGB invoices. This tool provides functionality to sign batch pay requests using extended private keys (XPRV) and includes a utility to derive XPRV keys from BIP39 mnemonic phrases.

## Features

- **Batch Pay Request Signing**: Sign batch payment requests using extended private keys (XPRV)
- **Mnemonic to XPRV Conversion**: Utility tool to derive XPRV keys from BIP39 mnemonic phrases
- **Secure Key Management**: Uses standard Python libraries for secure key handling
- **Multiple Network Support**: Supports mainnet, testnet, and regtest networks

## Prerequisites

- Python 3.7 or higher
- pip package manager

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd batch_invoice_demo/python
```

2. Create a virtual environment (recommended):
```bash
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

3. Install dependencies:
```bash
pip install -r requirements.txt
```

## Configuration

The main application requires environment variables to be set. You can create a `.env` file in the `python` directory with the following variables:

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
- `CORE_API_URL`: Core API endpoint URL

### Example `.env` File

```env
PUB_DESCRIPTOR=[9c734ef0/86h/827167h/0h]tpubDDpgp3MoRydYFXayCRRhkj7JUxnuECoVgHx61Po2jNYNhzk9xLyusYGmaH9S4shnQFH8EEDv8PkTcA75186TNEAXWUkAcrZY8ENmkjGe5En/<0;1>/*
XPRV=XPRV
API_KEY=your_api_key_here
INVOICE=your_invoice_string_here
FEE_RATE=5
CORE_API_URL=
```

## Usage

### Main Application: Batch Invoice Signing

The main application signs a batch pay request and outputs the signed request in JSON format along with an example curl command.

```bash
python python_batch_pay_demo.py
```

Or if using a virtual environment:

```bash
source venv/bin/activate  # On Windows: venv\Scripts\activate
python python_batch_pay_demo.py
```

The application will:
1. Load environment variables from `.env` file
2. Extract the derivation path from `PUB_DESCRIPTOR`
3. Derive the child private key from `XPRV` using the extracted path
4. Sign the batch pay request
5. Output:
   - The message to sign
   - The signature
   - The complete signed request in JSON format
   - An example curl command to submit the request

#### Example Output

```
Message to sign: {"fee_rate":5,"invoice":"your_invoice","timestamp":1234567890}
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
curl -X POST 'https://xxxxxxx.com/batch_pay' \
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
python mnemonic_to_xprv.py --mnemonic "word1 word2 word3 ... word12"
```

#### With Network Specification

```bash
python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12" --network regtest
```

Supported networks: `mainnet`, `testnet`, `regtest` (default: `regtest`)

#### With Derivation Path

Derive a child XPRV at a specific path:

```bash
python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12" --network regtest --path "m/86h/827167h/0h"
```

#### With Passphrase

Use a BIP39 passphrase:

```bash
python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12" --passphrase "your_passphrase"
```

#### Output XPUB Instead of XPRV

```bash
python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12" --xpub
```

#### Output Both XPRV and XPUB

```bash
python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12" --both
```

#### Complete Example

```bash
python mnemonic_to_xprv.py \
  --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --network regtest \
  --path "m/86h/827167h/0h" \
  --passphrase ""
```

## How It Works

### Signing Process

1. **Message Generation**: Creates a canonical JSON message string from the request parameters with sorted keys
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

### Message Format

The signing message is a canonical JSON representation of the request with the following structure:
```json
{
  "fee_rate": 5,
  "invoice": "invoice_string",
  "timestamp": 1234567890
}
```

The JSON is encoded with sorted keys and no extra whitespace to ensure deterministic signing.

## Security Considerations

- **Private Keys**: Never commit private keys or `.env` files to version control
- **XPRV Validation**: Ensure that `XPRV` is actually a private key (not a public key)
- **Environment Variables**: Store sensitive information in `.env` files and ensure they are in `.gitignore`
- **Virtual Environment**: Use a virtual environment to isolate dependencies

## Error Handling

The application provides detailed error messages for common issues:

- Missing environment variables
- Invalid descriptor format
- Invalid derivation path
- Invalid mnemonic phrase
- Network parsing errors

## Dependencies

Key dependencies include:
- `python-dotenv`: Environment variable management
- `ecdsa`: Elliptic curve digital signature algorithm
- `bip32`: BIP32 key derivation
- `mnemonic`: BIP39 mnemonic phrase handling

## Differences from Rust Version

- **Message Format**: The Python version uses canonical JSON with sorted keys, while the Rust version uses a custom format from the `SignMessage` trait. For production use, ensure the message format matches the server's expected format.
- **Library Differences**: Python uses `ecdsa` and `bip32` libraries, while Rust uses `bitcoin` crate
- **Signature Encoding**: Both versions produce compatible signatures, but the implementation details may differ

## License

[Specify your license here]

## Contributing

[Add contribution guidelines if applicable]
