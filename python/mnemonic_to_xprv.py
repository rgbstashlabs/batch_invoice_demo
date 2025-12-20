#!/usr/bin/env python3
"""Command line tool to derive XPRV/XPUB from BIP39 mnemonic phrase.

Usage examples:
    python mnemonic_to_xprv.py --mnemonic "word1 word2 ... word12"
    python mnemonic_to_xprv.py --mnemonic "word1 word2 ..." --network regtest --path "m/86h/827167h/0h"
    python mnemonic_to_xprv.py --mnemonic "word1 word2 ..." --passphrase "optional_passphrase"
    python mnemonic_to_xprv.py --mnemonic "word1 word2 ..." --xpub
    python mnemonic_to_xprv.py --mnemonic "word1 word2 ..." --both
"""

import argparse
import sys

from bip32 import BIP32
from mnemonic import Mnemonic


def derive_xprv_from_mnemonic(
    mnemonic_phrase: str,
    passphrase: str,
    network: str,
) -> BIP32:
    """Derive XPRV from BIP39 mnemonic phrase.

    Args:
        mnemonic_phrase: BIP39 mnemonic phrase (12 or 24 words)
        passphrase: BIP39 passphrase (optional, default is empty string)
        network: Network type: mainnet, testnet, or regtest

    Returns:
        BIP32 object with the root/master XPRV
    """
    # Clean mnemonic phrase: strip whitespace and remove trailing punctuation
    mnemonic_phrase = mnemonic_phrase.strip().rstrip(".,;:!?")

    # Validate mnemonic
    mnemo = Mnemonic("english")
    if not mnemo.check(mnemonic_phrase):
        word_count = len(mnemonic_phrase.split())
        raise ValueError(
            f"Invalid BIP39 mnemonic phrase. "
            f"Found {word_count} words (expected 12 or 24). "
            f"Please check for typos or extra characters."
        )

    # Generate seed from mnemonic using PBKDF2-HMAC-SHA512
    # seed = PBKDF2(HMAC-SHA512, password=mnemonic, salt="mnemonic" + passphrase, 2048, 64)
    seed = mnemo.to_seed(mnemonic_phrase, passphrase=passphrase)

    # Map network string to BIP32 network parameter
    # BIP32 library uses: "main" for mainnet, "test" for testnet/regtest
    bip32_network = "main" if network.lower() in ("mainnet", "bitcoin") else "test"

    # Generate root/master XPRV from seed
    bip32 = BIP32.from_seed(seed, network=bip32_network)

    return bip32


def network_from_str(name: str) -> str:
    """Validate and normalize network name."""
    name_lower = name.lower()
    if name_lower in ("mainnet", "bitcoin"):
        return "mainnet"
    elif name_lower == "testnet":
        return "testnet"
    elif name_lower == "regtest":
        return "regtest"
    else:
        raise ValueError(
            f"Unsupported network: {name}. Use mainnet/testnet/regtest"
        )


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        prog="mnemonic-to-xprv",
        description="Derive XPRV/XPUB from BIP39 mnemonic phrase",
    )
    parser.add_argument(
        "-m",
        "--mnemonic",
        required=True,
        help="BIP39 mnemonic phrase (12 or 24 words)",
    )
    parser.add_argument(
        "-n",
        "--network",
        default="regtest",
        choices=["mainnet", "testnet", "regtest", "bitcoin"],
        help="Network type: mainnet, testnet, or regtest (default: regtest)",
    )
    parser.add_argument(
        "-d",
        "--path",
        help="Derivation path (optional, e.g., m/86h/827167h/0h). "
        "If not provided, returns the root/master XPRV",
    )
    parser.add_argument(
        "-p",
        "--passphrase",
        default="",
        help="BIP39 passphrase (optional, default is empty string)",
    )
    parser.add_argument(
        "--xpub",
        action="store_true",
        help="Output XPUB instead of XPRV",
    )
    parser.add_argument(
        "--both",
        action="store_true",
        help="Output both XPRV and XPUB",
    )

    args = parser.parse_args()

    # Validate mutually exclusive options
    if args.xpub and args.both:
        parser.error("--xpub and --both cannot be used together")

    try:
        # Parse and validate network
        network = network_from_str(args.network)

        # Derive root/master XPRV from mnemonic
        root_bip32 = derive_xprv_from_mnemonic(
            args.mnemonic, args.passphrase, network
        )

        # Determine output mode
        output_xprv = not args.xpub or args.both
        output_xpub = args.xpub or args.both

        # If path is provided, derive child keys
        if args.path:
            # BIP32 library supports path strings directly (e.g., "m/86h/827167h/0h")
            # or lists of integers
            print(f"Network: {network}")
            print(f"Derivation Path: {args.path}")

            if output_xprv:
                child_xprv = root_bip32.get_xpriv_from_path(args.path)
                print(f"XPRV: {child_xprv}")

            if output_xpub:
                child_xpub = root_bip32.get_xpub_from_path(args.path)
                print(f"XPUB: {child_xpub}")
        else:
            # Get root/master keys
            print(f"Network: {network}")

            if output_xprv:
                root_xprv = root_bip32.get_xpriv()
                print(f"Root/Master XPRV: {root_xprv}")

            if output_xpub:
                root_xpub = root_bip32.get_xpub()
                print(f"Root/Master XPUB: {root_xpub}")

            if not args.xpub and not args.both:
                print("\nTo derive a child key, use --path option, e.g.:")
                print('  --path "m/86h/827167h/0h"')
                print("\nTo output XPUB, use --xpub or --both option")

    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        import traceback

        traceback.print_exc()
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
