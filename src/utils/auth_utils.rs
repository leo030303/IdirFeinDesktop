use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, TOTP};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub client_id: String,
    pub client_secret: Vec<u8>,
}

impl AuthCredentials {
    pub fn calculate_totp(&self) -> String {
        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, self.client_secret.clone())
            .expect("Invalid credentials");
        totp.generate_current().expect("Invalid credentials")
    }
}
