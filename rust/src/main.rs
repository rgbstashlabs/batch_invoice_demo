// Step 4: Alice calls Batch Pay
// When calling batch pay, you need to sign the parameters, an example of the signature code is shown below:

use base64::{Engine as _, engine::general_purpose};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
use chrono::Utc;
use dotenvy::dotenv;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

// Trait for signing messages
pub trait SignMessage {
    fn sign_message(&self, timestamp: i64) -> String;
}

#[derive(Debug)]
pub struct CoreApi {
    pub_descriptor: String,
    xprv: Secret<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayRequest {
    pub fee_rate: u32,
    pub invoice: String,
    pub coinselect: Option<RgbPayCoinSelect>,
}

// Implement SignMessage trait for PayRequest
impl SignMessage for PayRequest {
    fn sign_message(&self, timestamp: i64) -> String {
        // Create a canonical JSON representation for signing
        // This matches the Python implementation: convert to dict, add timestamp,
        // remove None fields, JSON-encode with sorted keys and no extra whitespace
        use serde_json::json;

        let mut data = json!({
            "fee_rate": self.fee_rate,
            "invoice": self.invoice,
            "timestamp": timestamp,
        });

        // Add coinselect if present
        if let Some(ref coinselect) = self.coinselect {
            data["coinselect"] = serde_json::to_value(coinselect).unwrap_or(serde_json::Value::Null);
        }

        // Serialize with sorted keys and compact format (no extra whitespace)
        serde_json::to_string(&data).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RgbPayCoinSelect {
    #[serde(skip_serializing_if = "Option::is_none")]
    outpoints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    psbt: Option<String>,
}

impl std::fmt::Display for RgbPayCoinSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignerBatchPayRequest {
    #[serde(
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub coinselect: Option<Option<Box<RgbPayCoinSelect>>>,
    #[serde(rename = "fee_rate")]
    pub fee_rate: u32,
    #[serde(rename = "invoice")]
    pub invoice: String,
    /// Hex String for binary data
    #[serde(rename = "signature")]
    pub signature: String,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

/// Extract derivation path from pub_descriptor
///
/// Descriptor format: [fingerprint/path]xpub/...
/// Example: [9c734ef0/86h/827167h/0h]tpubDDpgp3MoRydYFXayCRRhkj7JUxnuECoVgHx61Po2jNYNhzk9xLyusYGmaH9S4shnQFH8EEDv8PkTcA75186TNEAXWUkAcrZY8ENmkjGe5En/<0;1>/*
/// Returns the derivation path from the origin part in brackets
fn extract_derivation_path_from_descriptor(descriptor: &str) -> anyhow::Result<DerivationPath> {
    // Extract the origin path from brackets [fingerprint/path]
    let start_bracket = descriptor
        .find('[')
        .ok_or_else(|| anyhow::anyhow!("Descriptor missing opening bracket"))?;
    let end_bracket = descriptor
        .find(']')
        .ok_or_else(|| anyhow::anyhow!("Descriptor missing closing bracket"))?;

    let origin_str = &descriptor[start_bracket + 1..end_bracket];
    // Split by '/' and skip the first part (fingerprint), get the path parts
    let parts: Vec<&str> = origin_str.split('/').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid origin format in descriptor"));
    }

    // Reconstruct path from parts (skip fingerprint, use path parts)
    let path_parts: Vec<&str> = parts[1..].to_vec();
    let path_str = format!("m/{}", path_parts.join("/"));
    let path = DerivationPath::from_str(&path_str)?;

    Ok(path)
}

/// Signer wrapper that holds the request and performs signing
pub struct Signer<T> {
    inner: T,
    timestamp: i64,
    signature: Option<String>,
}

impl<T> Signer<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            timestamp: Utc::now().timestamp(),
            signature: None,
        }
    }

    pub fn sign(mut self, priv_key_bytes: &[u8]) -> anyhow::Result<Self>
    where
        T: SignMessage + Serialize,
    {
        // 1. Generate canonical message string using SignMessage trait
        let message = self.inner.sign_message(self.timestamp);
        println!("Message to sign: {}", message);

        // 2. Hash message with SHA256
        let digest = Sha256Hash::hash(message.as_bytes());

        // 3. Sign using secp256k1
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(priv_key_bytes)?;
        let msg = Message::from_digest_slice(digest.as_byte_array())?;
        let sig = secp.sign_ecdsa(&msg, &sk);

        // 4. Store signature as hex string
        self.signature = Some(hex::encode(sig.serialize_compact()));
        Ok(self)
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn signature(&self) -> Option<&String> {
        self.signature.as_ref()
    }
}

impl Signer<PayRequest> {
    /// Convert Signer<PayRequest> to SignerBatchPayRequest
    pub fn into_signed_request(self) -> SignerBatchPayRequest {
        SignerBatchPayRequest {
            coinselect: self.inner.coinselect.map(|cs| Some(Box::new(cs))),
            fee_rate: self.inner.fee_rate,
            invoice: self.inner.invoice,
            signature: self.signature.unwrap_or_default(),
            timestamp: self.timestamp,
        }
    }
}

impl CoreApi {
    pub fn sign_batch_pay_req_with_xprv(
        &self,
        req: PayRequest,
    ) -> anyhow::Result<String> {
        let ctx = Secp256k1::new();
        let mut signer = Signer::new(req);

        // Derive child private key from xprv
        // Xpriv::from_str automatically detects network from version bytes
        let xprv_str = self.xprv.expose_secret();

        // Check if it's a public key (xpub/zpub/tpub) instead of private key
        if xprv_str.starts_with("xpub") || xprv_str.starts_with("zpub") || xprv_str.starts_with("tpub") || xprv_str.starts_with("upub") {
            return Err(anyhow::anyhow!(
                "XPRV environment variable contains a public key ({}), but a private key (xprv/zprv/tprv/uprv) is required. Please provide the corresponding private key.",
                if xprv_str.len() > 20 { &xprv_str[..20] } else { xprv_str }
            ));
        }

        let extended_private_key = Xpriv::from_str(xprv_str)
            .map_err(|e| anyhow::anyhow!(
                "Failed to parse XPRV. Make sure it's a valid xprv/zprv/tprv/uprv format. Error: {}",
                e
            ))?;

        // Extract derivation path from descriptor
        let path = extract_derivation_path_from_descriptor(&self.pub_descriptor)?;

        let derived_extended_private_key = extended_private_key.derive_priv(&ctx, &path)?;

        // Sign the request
        signer = signer.sign(&derived_extended_private_key.private_key.secret_bytes())?;

        // Return signature
        Ok(signer.signature().unwrap().clone())
    }
}

fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenv().ok();

    // Read parameters from environment variables
    let pub_descriptor = env::var("PUB_DESCRIPTOR")
        .map_err(|_| anyhow::anyhow!("PUB_DESCRIPTOR environment variable not set"))?;

    let xprv = env::var("XPRV")
        .map_err(|_| anyhow::anyhow!("XPRV environment variable not set"))?;

    let api_key = env::var("API_KEY")
        .map_err(|_| anyhow::anyhow!("API_KEY environment variable not set"))?;

    let invoice = env::var("INVOICE")
        .map_err(|_| anyhow::anyhow!("INVOICE environment variable not set"))?;

    let fee_rate: u32 = env::var("FEE_RATE")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .map_err(|_| anyhow::anyhow!("FEE_RATE must be a valid number"))?;

    let core_api_url = env::var("CORE_API_URL").map_err(|_| anyhow::anyhow!("CORE_API_URL environment variable not set"))?;

    // Create PayRequest
    let pay_req = PayRequest {
        fee_rate,
        invoice: invoice.clone(),
        coinselect: None,
    };

    let core_api = CoreApi {
        pub_descriptor: pub_descriptor.clone(),
        xprv: Secret::new(xprv),
    };

    // Sign the request and get signature
    match core_api.sign_batch_pay_req_with_xprv(pay_req) {
        Ok(signature) => {
            // Generate authorization header as base64 of pub_descriptor
            let authorization = general_purpose::STANDARD.encode(pub_descriptor.as_bytes());

            // Create the signed request for display
            let timestamp = Utc::now().timestamp();
            let signed_req = SignerBatchPayRequest {
                coinselect: None,
                fee_rate,
                invoice: invoice.clone(),
                signature: signature.clone(),
                timestamp,
            };

            // Print the signature
            println!("Signature: {}", signature);

            // Print the JSON that can be sent to /batch_pay
            let json = serde_json::to_string_pretty(&signed_req)?;
            println!("\n=== Signed Batch Pay Request (JSON) ===");
            println!("{}", json);

            // Example curl command
            println!("\n=== Example curl command ===");
            println!(
                "curl -X POST '{}/batch_pay' \\", core_api_url
            );
            println!("  --header 'x-api-key: {}' \\", api_key);
            println!("  --header 'authorization: Bearer {}' \\", authorization);
            println!("  --header 'Content-Type: application/json' \\");
            println!("  --data-raw '{}'", serde_json::to_string(&signed_req)?);
        }
        Err(e) => {
            eprintln!("Error signing request: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
