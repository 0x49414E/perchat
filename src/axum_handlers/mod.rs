use axum::extract::{Json,State};
use crate::{AppState, Client};
use std::sync::{Arc, Mutex};
use serde_json::{json, Value, Error};
use uuid::Uuid;
use dotenv::dotenv;
use chacha20poly1305::{aead::{Aead, AeadCore, KeyInit, OsRng}, ChaCha20Poly1305, ChaChaPoly1305, Key, Nonce};
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::generic_array::typenum::Unsigned;

pub struct Decryptor {
    pub key : Key,
    pub nonce: Nonce,
}

impl Decryptor {
    pub fn new() -> Self {
        Self {
            key: ChaCha20Poly1305::generate_key(&mut OsRng),
            nonce: ChaCha20Poly1305::generate_nonce(&mut OsRng),
        }
    }
    pub fn encrypt(&self, cleartext: &str) -> String {
        let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&self.key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let mut obsf = cipher.encrypt(&nonce, cleartext.as_bytes()).unwrap();
        obsf.splice(..0, nonce.iter().copied());
        base64::encode(obsf)
    }

    pub fn decrypt(&self, obsf: String) -> String {
        let obsf = base64::decode(obsf).unwrap();
        type NonceSize = <ChaCha20Poly1305 as AeadCore>::NonceSize;
        let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&self.key));
        let (nonce, ciphertext) = obsf.split_at(NonceSize::to_usize());
        let nonce = GenericArray::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();
        String::from_utf8(plaintext).unwrap()
    }
}
pub async fn verify_id() {}

pub async fn generate_id(State(state) : State<Arc<AppState>>, key: String) -> Json<Value> {

    let id_priv = Uuid::new_v4().to_string();

    let server_key = (&state.server_key).clone();

    // s == id_priv

    let decryptor = state.decryptor.lock().unwrap().clone();

    let e = &decryptor.encrypt(id_priv.as_str());

    let s = &decryptor.decrypt(e.clone());

    println!("{}: {}", s, id_priv);

    let id_pub = e.clone();

    Json(serde_json::to_value(Client { id_pub, id_priv }).unwrap())

}