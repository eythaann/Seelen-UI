use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use seelen_core::system_state::SeelenSession;
use serde::Deserialize;

/// Full JWT payload from the Seelen auth service.
/// Internal only — never serialized to the frontend.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtPayload {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub plan: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
    /// Unix timestamp (seconds) at which this token expires.
    pub exp: u64,
}

impl JwtPayload {
    pub fn secs_until_expiry(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.exp.saturating_sub(now)
    }

    pub fn into_session(self) -> SeelenSession {
        SeelenSession {
            id: self.id,
            email: self.email,
            username: self.username,
            display_name: self.display_name,
            avatar: self.avatar,
            plan: self.plan,
            permissions: self.permissions,
            roles: self.roles,
        }
    }
}

/// Decodes the payload segment of a JWT without verifying the signature.
pub fn decode_jwt_payload(token: &str) -> crate::error::Result<JwtPayload> {
    let mut parts = token.splitn(3, '.');
    parts.next(); // header
    let payload_b64 = parts.next().ok_or("Invalid JWT: missing payload segment")?;
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| "Invalid JWT: could not base64-decode payload")?;
    let payload: JwtPayload = serde_json::from_slice(&payload_bytes)?;
    Ok(payload)
}
