use ed25519_dalek::Verifier;
use ed25519_dalek::{PublicKey, Signature};
use std::str::FromStr;

pub fn verify_signature(
    public_key: PublicKey,
    signature: String,
    timestamp: String,
    body: String,
) -> bool {

    let signature = Signature::from_str(signature.as_str());
    let signature = match signature {
        Ok(signature) => signature,
        Err(_) => return false
    };

    let msg = format!("{timestamp}{body}");
    let verified = public_key.verify(msg.as_bytes(), &signature);

    verified.is_ok()
}
