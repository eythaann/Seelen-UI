use std::{collections::HashMap, sync::LazyLock};

use parking_lot::RwLock;
use uuid::Uuid;

static TOKENS: LazyLock<RwLock<WebviewTokens>> = LazyLock::new(Default::default);

/// Per webview tokens used to authenticate the events websocket.
/// Tokens are never removed: a webview reopened with the same label reuses its token.
#[derive(Default)]
struct WebviewTokens {
    by_label: HashMap<String, Uuid>,
    by_token: HashMap<Uuid, String>,
}

/// Returns the token of the webview with the given raw label, creating it on first use.
pub fn get_or_create(label: &str) -> Uuid {
    if let Some(token) = TOKENS.read().by_label.get(label) {
        return *token;
    }

    let mut tokens = TOKENS.write();
    if let Some(token) = tokens.by_label.get(label) {
        return *token;
    }

    let token = Uuid::new_v4();
    tokens.by_label.insert(label.to_owned(), token);
    tokens.by_token.insert(token, label.to_owned());
    token
}

/// Returns the raw label of the webview owning the token.
pub fn resolve(token: &Uuid) -> Option<String> {
    TOKENS.read().by_token.get(token).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_reused_per_label() {
        let first = get_or_create("tokens-test-a");
        assert_eq!(first, get_or_create("tokens-test-a"));
        assert_ne!(first, get_or_create("tokens-test-b"));
        assert_eq!(resolve(&first).as_deref(), Some("tokens-test-a"));
        assert_eq!(resolve(&Uuid::new_v4()), None);
    }
}
