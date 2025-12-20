// Command line tool to derive XPRV from BIP39 mnemonic phrase
// Usage examples:
//   cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ... word12"
//   cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ..." --network regtest --path "m/86h/827167h/0h"
//   cargo run --bin mnemonic-to-xprv -- --mnemonic "word1 word2 ..." --passphrase "optional_passphrase"

use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::Network;
use clap::Parser;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(name = "mnemonic-to-xprv")]
#[command(about = "Derive XPRV from BIP39 mnemonic phrase")]
struct Args {
    /// BIP39 mnemonic phrase (12 or 24 words)
    #[arg(short, long)]
    mnemonic: String,

    /// Network type: mainnet, testnet, or regtest
    #[arg(short, long, default_value = "regtest")]
    network: String,

    /// Derivation path (optional, e.g., m/86h/827167h/0h)
    /// If not provided, returns the root/master XPRV
    #[arg(short = 'd', long)]
    path: Option<String>,

    /// BIP39 passphrase (optional, default is empty string)
    #[arg(short = 'p', long, default_value = "")]
    passphrase: String,
}

fn derive_xprv_from_mnemonic(
    mnemonic: &str,
    passphrase: &str,
    network: Network,
) -> anyhow::Result<Xpriv> {
    // Generate seed from mnemonic using PBKDF2-HMAC-SHA512
    // seed = PBKDF2(HMAC-SHA512, password=mnemonic, salt="mnemonic" + passphrase, 2048, 64)
    type HmacSha512 = Hmac<Sha512>;
    let salt = format!("mnemonic{}", passphrase);
    let mut seed = [0u8; 64];
    pbkdf2::<HmacSha512>(mnemonic.as_bytes(), salt.as_bytes(), 2048, &mut seed)
        .map_err(|e| anyhow::anyhow!("Failed to generate seed from mnemonic: {e}"))?;

    // Generate root/master XPRV from seed
    let xprv = Xpriv::new_master(network, &seed)
        .map_err(|e| anyhow::anyhow!("Failed to derive master xprv: {e}"))?;

    Ok(xprv)
}

fn network_from_str(name: &str) -> anyhow::Result<Network> {
    match name.to_lowercase().as_str() {
        "mainnet" | "bitcoin" => Ok(Network::Bitcoin),
        "testnet" => Ok(Network::Testnet),
        "regtest" => Ok(Network::Regtest),
        other => Err(anyhow::anyhow!(
            "Unsupported network: {other}. Use mainnet/testnet/regtest"
        )),
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Parse network
    let network = network_from_str(&args.network)?;

    // Derive root/master XPRV from mnemonic
    let root_xprv = derive_xprv_from_mnemonic(&args.mnemonic, &args.passphrase, network)?;

    // If path is provided, derive child XPRV
    if let Some(path_str) = args.path {
        let path = DerivationPath::from_str(&path_str)?;
        let ctx = bitcoin::secp256k1::Secp256k1::new();
        let derived_xprv = root_xprv.derive_priv(&ctx, &path)?;

        println!("Network: {}", args.network);
        println!("Derivation Path: {}", path_str);
        println!("XPRV: {}", derived_xprv);
    } else {
        println!("Network: {}", args.network);
        println!("Root/Master XPRV: {}", root_xprv);
        println!("\nTo derive a child XPRV, use --path option, e.g.:");
        println!("  --path \"m/86h/827167h/0h\"");
    }

    Ok(())
}
