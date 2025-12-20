"""Python demo equivalent of the Rust batch-pay signing example.

This script roughly mirrors `rust/src/main.rs`:
- Reads `PUB_DESCRIPTOR`, `XPRV`, `API_KEY`, `INVOICE`, `FEE_RATE` from environment (or `.env`).
- Extracts the derivation path from `PUB_DESCRIPTOR`.
- Derives a child private key from the extended private key (XPRV).
- Builds a canonical message from the pay request + timestamp.
- Signs it with secp256k1 over SHA256.
- Prints the signature, the JSON body you can POST to `/batch_pay`,
  and a sample `curl` command.

NOTE:
- Comments are in English by request.
- The exact message format used in the Rust macro `#[derive(Signer)]`
  is not public here, so this demo uses a reasonable canonical JSON
  representation. For production use, you must ensure the message
  built here matches the server's expected format.

Dependencies (example `requirements.txt`):

    python-dotenv
    ecdsa
    bip32

Install with:

    pip install python-dotenv ecdsa bip32

"""

import base64
import hashlib
import json
import os
from dataclasses import dataclass, asdict
from typing import List, Optional

from dotenv import load_dotenv
from bip32 import BIP32
from ecdsa import SECP256k1, SigningKey


# ----------------------------- Data models -----------------------------


@dataclass
class RgbPayCoinSelect:
    # Only one of these would normally be used
    outpoints: Optional[List[str]] = None
    psbt: Optional[str] = None


@dataclass
class PayRequest:
    fee_rate: int
    invoice: str
    coinselect: Optional[RgbPayCoinSelect] = None


@dataclass
class SignerBatchPayRequest:
    coinselect: Optional[Optional[RgbPayCoinSelect]]
    fee_rate: int
    invoice: str
    signature: str
    timestamp: int


# ------------------------ Helper / utility code ------------------------


def extract_derivation_path_from_descriptor(descriptor: str) -> str:
    """Extracts a derivation path string from a descriptor.

    Rust version comment:
        Descriptor format: [fingerprint/path]xpub/...
        Example: [9c734ef0/86h/827167h/0h]tpub.../<0;1>/*

    This function returns a standard BIP32-style path string like:
        "m/86h/827167h/0h"
    """

    start = descriptor.find("[")
    if start == -1:
        raise ValueError("Descriptor missing opening bracket '['")

    end = descriptor.find("]", start)
    if end == -1:
        raise ValueError("Descriptor missing closing bracket '']'")

    origin = descriptor[start + 1 : end]
    parts = origin.split("/")
    if len(parts) < 2:
        raise ValueError("Invalid origin format in descriptor")

    # Skip the first element (fingerprint), keep the path parts
    path_parts = parts[1:]
    path_str = "m/" + "/".join(path_parts)
    return path_str


def parse_bip32_path(path: str) -> List[int]:
    """Convert a BIP32 path like "m/86h/827167h/0h" to a list of indexes.

    We treat suffixes 'h' or "'" as hardened (add 0x80000000).
    """

    if not path.startswith("m/"):
        raise ValueError("Path must start with 'm/'")

    result: List[int] = []
    for elem in path[2:].split("/"):
        if not elem:
            continue
        hardened = elem.endswith("h") or elem.endswith("'")
        if hardened:
            elem = elem[:-1]
        index = int(elem)
        if index < 0:
            raise ValueError("Path index cannot be negative")
        if hardened:
            index |= 0x80000000
        result.append(index)
    return result


def derive_child_privkey_from_xprv(xprv: str, path: str) -> bytes:
    """Derive child private key bytes from an extended private key and path.

    This is a Python rough equivalent of the Rust logic using `Xpriv`.

    NOTE: bip32 library uses `from_xpriv` instead of `from_xprv`.
    """

    bip32 = BIP32.from_xpriv(xprv)
    indexes = parse_bip32_path(path)
    privkey = bip32.get_privkey_from_path(indexes)
    return privkey


def build_signing_message(req: PayRequest, timestamp: int) -> str:
    """Build the canonical message string to sign.

    We mimic a stable JSON representation:
    - Convert `PayRequest` to dict
    - Add `timestamp`
    - JSON-encode with sorted keys and no extra whitespace.
    """

    data = asdict(req)
    data["timestamp"] = timestamp
    # Remove None fields explicitly for compactness
    data = {k: v for k, v in data.items() if v is not None}
    message = json.dumps(data, sort_keys=True, separators=(",", ":"))
    return message


def sign_message_secp256k1_sha256(message: str, privkey_bytes: bytes) -> str:
    """Sign a message (string) with SHA256 + secp256k1 ECDSA.

    Returns a compact 64-byte signature hex string.
    """

    digest = hashlib.sha256(message.encode("utf-8")).digest()

    sk = SigningKey.from_string(privkey_bytes, curve=SECP256k1)
    # `sigencode_string` gives raw 64-byte R||S
    signature_bytes = sk.sign_digest(digest, sigencode=lambda r, s, order: r.to_bytes(32, "big") + s.to_bytes(32, "big"))
    return signature_bytes.hex()


# ------------------------------ Main logic -----------------------------


def main() -> None:
    # Load .env if present
    load_dotenv()

    pub_descriptor = os.getenv("PUB_DESCRIPTOR")
    xprv = os.getenv("XPRV")
    api_key = os.getenv("API_KEY")
    invoice = os.getenv("INVOICE")
    fee_rate_str = os.getenv("FEE_RATE", "5")
    core_api_url = os.getenv("CORE_API_URL", "")

    if not pub_descriptor:
        raise RuntimeError("PUB_DESCRIPTOR environment variable not set")
    if not xprv:
        raise RuntimeError("XPRV environment variable not set")
    if not api_key:
        raise RuntimeError("API_KEY environment variable not set")
    if not invoice:
        raise RuntimeError("INVOICE environment variable not set")

    try:
        fee_rate = int(fee_rate_str)
    except ValueError as exc:
        raise RuntimeError("FEE_RATE must be a valid integer") from exc

    # Build the unsigned pay request (no coinselect in this simple example)
    pay_req = PayRequest(fee_rate=fee_rate, invoice=invoice, coinselect=None)

    # Derive child private key from XPRV using the path from descriptor
    path = extract_derivation_path_from_descriptor(pub_descriptor)
    child_privkey = derive_child_privkey_from_xprv(xprv, path)

    # Build message and sign
    import time

    timestamp = int(time.time())
    message = build_signing_message(pay_req, timestamp)
    print(f"Message to sign: {message}")

    signature_hex = sign_message_secp256k1_sha256(message, child_privkey)
    print(f"Signature: {signature_hex}")

    # Build the request body equivalent to `SignerBatchPayRequest`
    signed_req = SignerBatchPayRequest(
        coinselect=None,
        fee_rate=fee_rate,
        invoice=invoice,
        signature=signature_hex,
        timestamp=timestamp,
    )

    signed_req_json = json.dumps(asdict(signed_req), indent=2, sort_keys=False)

    # Authorization header is base64 of pub_descriptor (same as Rust example)
    authorization = base64.b64encode(pub_descriptor.encode("utf-8")).decode("ascii")

    print("\n=== Signed Batch Pay Request (JSON) ===")
    print(signed_req_json)

    print("\n=== Example curl command ===")
    print(
        f"curl -X POST '{core_api_url}/batch_pay' \\",
    )
    print(f"  --header 'x-api-key: {api_key}' \\")
    print(f"  --header 'authorization: Bearer {authorization}' \\")
    print("  --header 'Content-Type: application/json' \\")
    print(
        "  --data-raw '" + json.dumps(asdict(signed_req), separators=(",", ":")) + "'",
    )


if __name__ == "__main__":
    main()
