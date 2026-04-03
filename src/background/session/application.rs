use std::sync::LazyLock;

use parking_lot::Mutex;
use seelen_core::system_state::SeelenSession;
use uuid::Uuid;
use windows::{
    core::HSTRING,
    Security::Credentials::{PasswordCredential, PasswordVault},
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{decode_jwt_payload, JwtPayload};

// ─── Shared HTTP client ───────────────────────────────────────────────────────

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("failed to build reqwest client")
});

// ─── Constants ────────────────────────────────────────────────────────────────

#[cfg(dev)]
const AUTH_BASE_URL: &str = "https://auth.local.seelen.io";
#[cfg(not(dev))]
const AUTH_BASE_URL: &str = "https://auth.seelen.io";

#[cfg(dev)]
const WEBSITE_BASE_URL: &str = "https://local.seelen.io";
#[cfg(not(dev))]
const WEBSITE_BASE_URL: &str = "https://seelen.io";

/// Credential Manager resource identifier used for all Seelen auth tokens.
/// Dev and production builds use different keys so their sessions don't collide.
#[cfg(dev)]
const CREDENTIAL_RESOURCE: &str = "SeelenUI:auth:dev";
#[cfg(not(dev))]
const CREDENTIAL_RESOURCE: &str = "SeelenUI:auth";
const ACCESS_TOKEN_KEY: &str = "access_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";

/// Start refreshing 2 minutes before the access token expires.
const REFRESH_BUFFER_SECS: u64 = 120;
/// Delays between retry attempts after a transient refresh failure (network errors, etc.).
const REFRESH_RETRY_DELAYS_SECS: &[u64] = &[30, 120, 300];

// ─── Events ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SessionManagerEvent {
    Changed(Option<SeelenSession>),
}

event_manager!(SessionManager, SessionManagerEvent);

// ─── Manager ──────────────────────────────────────────────────────────────────

pub struct SessionManager {
    pub session: Option<SeelenSession>,
    /// CSRF state value set when `login()` opens the browser. Consumed by
    /// `handle_auth_callback()` to prevent open-redirect / replay attacks.
    pub pending_state: Option<String>,
}

impl SessionManager {
    pub fn instance() -> &'static Mutex<SessionManager> {
        static INSTANCE: LazyLock<Mutex<SessionManager>> =
            LazyLock::new(|| Mutex::new(SessionManager::load()));
        &INSTANCE
    }

    /// Synchronous init: reads persisted tokens and restores the session.
    /// If the token is expired (or missing), `schedule_refresh(0)` fires
    /// immediately so a refresh is attempted in the background right away;
    /// any cached session data remains available for offline use in the meantime.
    fn load() -> Self {
        match Self::read_stored_session() {
            Ok((session, payload)) => {
                Self::schedule_refresh(payload.secs_until_expiry());
                SessionManager {
                    session: Some(session),
                    pending_state: None,
                }
            }
            Err(_) => {
                // No stored tokens — user must log in explicitly.
                SessionManager {
                    session: None,
                    pending_state: None,
                }
            }
        }
    }

    pub fn has_premium_access(&self) -> bool {
        self.session
            .as_ref()
            .map(|s| s.permissions.contains(&"resource-premium".to_string()))
            .unwrap_or(false)
    }

    /// Reads the stored access token and returns the decoded session.
    /// Returns the session even if the token is expired so the app can operate
    /// offline with cached user data; the caller is responsible for scheduling
    /// a background refresh.
    fn read_stored_session() -> Result<(SeelenSession, JwtPayload)> {
        let token = Self::read_credential(ACCESS_TOKEN_KEY)?;
        let payload = decode_jwt_payload(&token)?;
        let session = payload.into_session();
        // Re-decode to keep both the session and the payload (needed for exp).
        let payload2 = decode_jwt_payload(&token)?;
        Ok((session, payload2))
    }

    // ─── Windows Credential Manager ───────────────────────────────────────────

    fn store_credential(key: &str, value: &str) -> Result<()> {
        let resource = HSTRING::from(CREDENTIAL_RESOURCE);
        let user_name = HSTRING::from(key);
        let password = HSTRING::from(value);
        let vault = PasswordVault::new()?;
        // Remove stale entry first so Add always succeeds.
        if let Ok(existing) = vault.Retrieve(&resource, &user_name) {
            vault.Remove(&existing).ok();
        }
        let cred = PasswordCredential::CreatePasswordCredential(&resource, &user_name, &password)?;
        vault.Add(&cred)?;
        Ok(())
    }

    pub fn read_credential(key: &str) -> Result<String> {
        let resource = HSTRING::from(CREDENTIAL_RESOURCE);
        let user_name = HSTRING::from(key);
        let vault = PasswordVault::new()?;
        let cred = vault.Retrieve(&resource, &user_name)?;
        cred.RetrievePassword()?;
        Ok(cred.Password()?.to_string())
    }

    fn delete_all_credentials() {
        let Ok(vault) = PasswordVault::new() else {
            return;
        };
        let resource = HSTRING::from(CREDENTIAL_RESOURCE);
        for key in [ACCESS_TOKEN_KEY, REFRESH_TOKEN_KEY] {
            let user_name = HSTRING::from(key);
            if let Ok(cred) = vault.Retrieve(&resource, &user_name) {
                vault.Remove(&cred).log_error();
            }
        }
    }

    // ─── Token refresh ────────────────────────────────────────────────────────

    /// Spawns a Tokio task that sleeps until `secs_until_expiry - REFRESH_BUFFER_SECS`
    /// and then delegates to `run_refresh_cycle`.
    fn schedule_refresh(secs_until_expiry: u64) {
        let sleep_secs = secs_until_expiry.saturating_sub(REFRESH_BUFFER_SECS);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)).await;
            SessionManager::run_refresh_cycle().await;
        });
    }

    /// Retry loop: attempts `refresh_tokens`, handles transient vs hard failures,
    /// updates manager state, emits events, and reschedules on success.
    ///
    /// Only a clear server-side rejection (non-2xx) triggers an immediate logout;
    /// connectivity problems alone never clear the session.
    async fn run_refresh_cycle() {
        let attempts = REFRESH_RETRY_DELAYS_SECS
            .iter()
            .copied()
            .chain(std::iter::once(0u64));

        for (attempt, delay) in attempts.enumerate() {
            match Self::refresh_tokens().await {
                Ok((session, secs)) => {
                    Self::instance().lock().session = Some(session.clone());
                    Self::send(SessionManagerEvent::Changed(Some(session)));
                    Self::schedule_refresh(secs);
                    return;
                }
                // Credentials deleted inside `refresh_tokens` on server rejection — log out.
                Err(ref e) if e.to_string().contains("Token refresh rejected") => {
                    log::warn!("Session refresh rejected by server, logging out: {e:?}");
                    Self::instance().lock().session = None;
                    Self::send(SessionManagerEvent::Changed(None));
                    return;
                }
                Err(e) => {
                    let remaining = REFRESH_RETRY_DELAYS_SECS.len().saturating_sub(attempt);
                    if remaining == 0 {
                        // All retries exhausted — keep credentials so the next launch retries.
                        log::warn!(
                            "Session auto-refresh failed after all retries, will retry on next launch: {e:?}"
                        );
                        return;
                    }
                    log::warn!(
                        "Session auto-refresh failed (attempt {}/{}, retrying in {delay}s): {e:?}",
                        attempt + 1,
                        REFRESH_RETRY_DELAYS_SECS.len(),
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                }
            }
        }
    }

    /// Calls the auth service refresh endpoint using the stored refresh token
    /// (sent as a manually constructed Cookie header, since there is no browser
    /// cookie jar in a desktop context).
    ///
    /// Pure HTTP operation: reads credentials, calls the endpoint, stores the new
    /// tokens, and returns the resulting session and its expiry. Does not touch
    /// manager state or schedule anything.
    async fn refresh_tokens() -> Result<(SeelenSession, u64)> {
        let refresh_token = Self::read_credential(REFRESH_TOKEN_KEY)
            .map_err(|_| "No stored refresh token — user must log in again")?;

        let response = HTTP_CLIENT
            .post(format!("{AUTH_BASE_URL}/v1/auth/refresh"))
            .header("Cookie", format!("refresh_token={refresh_token}"))
            .send()
            .await?;

        if !response.status().is_success() {
            Self::delete_all_credentials();
            return Err(format!("Token refresh rejected ({})", response.status()).into());
        }

        let (access_token, maybe_refresh_token) = extract_tokens_from_response(&response)?;

        Self::store_credential(ACCESS_TOKEN_KEY, &access_token)?;
        if let Some(ref rt) = maybe_refresh_token {
            Self::store_credential(REFRESH_TOKEN_KEY, rt)?;
        }

        let payload = decode_jwt_payload(&access_token)?;
        let secs = payload.secs_until_expiry();
        let session = payload.into_session();
        Ok((session, secs))
    }

    // ─── Public API ───────────────────────────────────────────────────────────

    /// Opens the system browser to the Seelen website sign-in page. After the
    /// user authenticates, the website redirects back via the deep-link scheme
    /// and `handle_auth_callback` completes the flow.
    pub fn login() -> Result<()> {
        let state = Uuid::new_v4().to_string();
        Self::instance().lock().pending_state = Some(state.clone());
        let url = format!(
            "{WEBSITE_BASE_URL}/signin?app_redirect={}&state={state}",
            urlencoding::encode("seelen-ui.uri://auth/callback"),
        );
        open::that(url)?;
        Ok(())
    }

    /// Called when the app receives the deep-link callback after browser login.
    /// `code` is an "app"-type refresh token from the website's `authorize-app`
    /// endpoint; `state` is the CSRF token generated by `login()`.
    pub async fn handle_auth_callback(code: String, state: String) -> Result<()> {
        {
            let mut manager = Self::instance().lock();
            let pending = manager.pending_state.take();
            // `pending_state` is in-memory only. If the app was restarted between
            // the `login()` call and the browser callback arriving, it will be None
            // and this check will fail even though the state strings match. The fix
            // is to complete the login flow without restarting the app in between.
            if pending.as_deref() != Some(state.as_str()) {
                return Err(
                    "Invalid state parameter in auth callback — possible CSRF attempt".into(),
                );
            }
        }

        // Exchange the app refresh token for a short-lived access token.
        let response = HTTP_CLIENT
            .post(format!("{AUTH_BASE_URL}/v1/auth/refresh"))
            .header("Cookie", format!("refresh_token={code}"))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("App auth callback failed ({})", response.status()).into());
        }

        let (access_token, maybe_refresh_token) = extract_tokens_from_response(&response)?;

        Self::store_credential(ACCESS_TOKEN_KEY, &access_token)?;
        // If the server didn't rotate the refresh token, keep the original code.
        let refresh_to_store = maybe_refresh_token.as_deref().unwrap_or(code.as_str());
        Self::store_credential(REFRESH_TOKEN_KEY, refresh_to_store)?;

        let payload = decode_jwt_payload(&access_token)?;
        let secs = payload.secs_until_expiry();
        let session = payload.into_session();

        Self::instance().lock().session = Some(session.clone());
        Self::schedule_refresh(secs);
        SessionManager::send(SessionManagerEvent::Changed(Some(session)));
        Ok(())
    }

    /// Calls the logout endpoint (best-effort) and wipes all stored tokens.
    pub async fn logout() -> Result<()> {
        // Best-effort server-side invalidation.
        if let Ok(refresh_token) = Self::read_credential(REFRESH_TOKEN_KEY) {
            let _ = HTTP_CLIENT
                .post(format!("{AUTH_BASE_URL}/v1/auth/logout"))
                .header("Cookie", format!("refresh_token={refresh_token}"))
                .send()
                .await;
        }

        Self::delete_all_credentials();
        Self::instance().lock().session = None;
        SessionManager::send(SessionManagerEvent::Changed(None));
        Ok(())
    }

    /// Returns the stored access token. Intended exclusively for internal background use.
    pub fn get_access_token() -> Result<String> {
        Self::read_credential(ACCESS_TOKEN_KEY)
    }

    /// Returns a GET request builder with `Authorization: Bearer <token>` set if a
    /// session is active. Use this for all authenticated background HTTP requests.
    pub fn authed_get(url: &str) -> reqwest::RequestBuilder {
        let mut req = HTTP_CLIENT.get(url);
        if let Ok(token) = Self::get_access_token() {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        req
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extracts the access token from the `Authorization` response header and,
/// optionally, a rotated refresh token from `Set-Cookie`.
fn extract_tokens_from_response(response: &reqwest::Response) -> Result<(String, Option<String>)> {
    let access_token = response
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or("Auth service response missing Authorization header")?
        .to_string();

    let refresh_token = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            s.split(';')
                .next()
                .and_then(|p| p.trim().strip_prefix("refresh_token="))
                .map(str::to_string)
        });

    Ok((access_token, refresh_token))
}
