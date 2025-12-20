# Batch Invoice Demo

A multi-language reference implementation for signing and submitting batch payment requests for Bitcoin/RGB invoices. This project provides both **Python** and **Rust** implementations of the same functionality, allowing developers to choose the language that best fits their needs.

## Overview

This project demonstrates how to:
- Sign batch payment requests using extended private keys (XPRV)
- Derive XPRV keys from BIP39 mnemonic phrases
- Submit signed requests to the batch pay API endpoint

Both implementations provide equivalent functionality, with language-specific optimizations and features.

## Features

- **Batch Pay Request Signing**: Sign batch payment requests using extended private keys (XPRV)
- **Mnemonic to XPRV Conversion**: Utility tool to derive XPRV keys from BIP39 mnemonic phrases
- **Secure Key Management**: Secure handling of private keys
- **Multiple Network Support**: Supports mainnet, testnet, and regtest networks
- **Dual Implementation**: Available in both Python and Rust

## Project Structure

```
batch_invoice_demo/
├── python/          # Python implementation
│   ├── python_batch_pay_demo.py
│   ├── mnemonic_to_xprv.py
│   ├── requirements.txt
│   └── README.md
├── rust/            # Rust implementation
│   ├── src/
│   ├── Cargo.toml
│   └── README.md
└── README.md        # This file
```

## Quick Start

### Choose Your Language

- **[Python Implementation](python/README.md)** - Recommended for rapid prototyping and integration with Python ecosystems
- **[Rust Implementation](rust/README.md)** - Recommended for production systems requiring maximum performance and security

### Common Configuration

Both implementations use the same environment variables. Create a `.env` file in the respective directory (`python/` or `rust/`):

```env
PUB_DESCRIPTOR=[9c734ef0/86h/827167h/0h]tpubDDpgp3MoRydYFXayCRRhkj7JUxnuECoVgHx61Po2jNYNhzk9xLyusYGmaH9S4shnQFH8EEDv8PkTcA75186TNEAXWUkAcrZY8ENmkjGe5En/<0;1>/*
XPRV=your_extended_private_key_here
API_KEY=your_api_key_here
INVOICE=your_invoice_string_here
FEE_RATE=5
CORE_API_URL=
```

### Python Quick Start

```bash
cd python
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt
python python_batch_pay_demo.py
```

See [Python README](python/README.md) for detailed documentation.

### Rust Quick Start

```bash
cd rust
cargo build --release
cargo run
```

See [Rust README](rust/README.md) for detailed documentation.

## Implementation Comparison

| Feature | Python | Rust |
|---------|--------|------|
| **Language** | Python 3.7+ | Rust (latest stable) |
| **Dependencies** | `python-dotenv`, `ecdsa`, `bip32`, `mnemonic` | `bitcoin`, `bitlight-core-derive`, `serde`, `chrono`, `secrecy` |
| **Message Format** | Canonical JSON with sorted keys | Custom format via `SignMessage` trait |
| **Performance** | Good for scripting and integration | Optimized for production |
| **Security** | Standard Python libraries | `secrecy` crate for secure key handling |
| **Use Case** | Rapid development, Python ecosystems | Production systems, maximum security |
| **XPUB Support** | Yes (`--xpub`, `--both` flags) | Root/master XPRV only |

### Key Differences

1. **Message Format**:
   - Python uses canonical JSON with sorted keys
   - Rust uses a custom format from the `SignMessage` trait
   - **Important**: For production use, ensure the message format matches the server's expected format

2. **Utility Features**:
   - Python version supports `--xpub` and `--both` flags for outputting public keys
   - Rust version focuses on XPRV derivation

3. **Security**:
   - Rust uses the `secrecy` crate to prevent accidental exposure of private keys
   - Python uses standard library practices for secure key handling

## Common Usage Examples

### Derive XPRV from Mnemonic

**Python:**
```bash
cd python
python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12" --network regtest --path "m/86h/827167h/0h"
```

**Rust:**
```bash
cd rust
cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ... word12" --network regtest --path "m/86h/827167h/0h"
```

### Sign and Submit Batch Pay Request

Both implementations follow the same workflow:
1. Load environment variables from `.env`
2. Extract derivation path from `PUB_DESCRIPTOR`
3. Derive child private key from `XPRV`
4. Sign the batch pay request
5. Output signed request and curl command

## How It Works

### Signing Process

Both implementations follow a similar signing process:

1. **Message Generation**: Create a canonical message from request parameters
2. **Hashing**: Hash the message with SHA256
3. **Key Derivation**: Extract derivation path from `PUB_DESCRIPTOR` and derive child private key from `XPRV`
4. **Signing**: Sign the hash using secp256k1 ECDSA
5. **Encoding**: Encode the signature as a hexadecimal string

### Descriptor Format

The `PUB_DESCRIPTOR` must follow the format:
```
[fingerprint/path]xpub/...
```

The derivation path is extracted from the part in brackets. For example:
- `[9c734ef0/86h/827167h/0h]tpub...` extracts path `m/86h/827167h/0h`

## Security Considerations

- **Private Keys**: Never commit private keys or `.env` files to version control
- **XPRV Validation**: Ensure that `XPRV` is actually a private key (not a public key like xpub/zpub/tpub/upub)
- **Environment Variables**: Store sensitive information in `.env` files and ensure they are in `.gitignore`
- **Virtual Environment** (Python): Use a virtual environment to isolate dependencies
- **Secure Storage** (Rust): Uses the `secrecy` crate to prevent accidental exposure of private keys

## Error Handling

Both implementations provide detailed error messages for:
- Missing environment variables
- Invalid descriptor format
- Invalid derivation path
- Invalid mnemonic phrase (Python) or XPRV format (Rust)
- Network parsing errors

## Prerequisites

### Python Implementation
- Python 3.7 or higher
- pip package manager

### Rust Implementation
- Rust (latest stable version)
- Cargo package manager
- Access to the `bitlight-core-derive` crate (located at `../bitlight-core-derive`)

## Contributing

Contributions are welcome! Please ensure that:
- Both Python and Rust implementations maintain feature parity
- Code follows the existing style and conventions
- Security best practices are maintained
- Documentation is updated accordingly

## License

[Specify your license here]

## Additional Resources

- [Python Implementation Documentation](python/README.md)
- [Rust Implementation Documentation](rust/README.md)
- [BIP32 Specification](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
- [BIP39 Specification](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
