use crate::resources::RESOURCES;
use salvo::prelude::*;

/// "SLU" interpreted as a base-36 number.
pub const LOCAL_API_PORT: u16 = 37_074;

const SCALAR_HTML: &str = include_str!("./scalar.html");

/// Serves the Scalar API reference page. We render this ourselves instead of using
/// `salvo_oapi::scalar::Scalar` so we can load the official Scalar CDN script directly
/// and freely configure it (theme, GitHub link, etc.) in `scalar.html`.
#[handler]
async fn scalar_docs(res: &mut Response) {
    res.render(Text::Html(SCALAR_HTML));
}

#[derive(serde::Serialize, salvo::oapi::ToSchema)]
struct PingResponse {
    name: &'static str,
    version: &'static str,
}

/// Ping
///
/// Returns the name and version of the application.
#[endpoint]
async fn ping() -> Json<PingResponse> {
    Json(PingResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Themes
///
/// Returns a list of all available themes.
#[endpoint(tag("Resources"))]
async fn themes() -> Json<Vec<std::sync::Arc<seelen_core::state::Theme>>> {
    Json(RESOURCES.themes())
}

/// Icon Packs
///
/// Returns a list of all available icon packs.
#[endpoint(tag("Resources"))]
async fn icon_packs() -> Json<Vec<std::sync::Arc<seelen_core::state::IconPack>>> {
    Json(RESOURCES.icon_packs())
}

/* #[endpoint]
async fn settings() -> Json<seelen_core::state::Settings> {
    let state = crate::state::application::FULL_STATE.load();
    Json(&state.settings)
} */

/// Starts the background HTTP server. Intended to be spawned once at app startup.
pub async fn start_server() {
    let api = Router::new()
        .push(Router::with_path("ping").get(ping))
        .push(
            Router::with_path("resources")
                .push(Router::with_path("themes").get(themes))
                .push(Router::with_path("icon-packs").get(icon_packs)),
        );

    let doc = OpenApi::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).merge_router(&api);

    let router = Router::new()
        .push(api)
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(Router::with_path("api-doc").goal(scalar_docs));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], LOCAL_API_PORT));
    let acceptor = match TcpListener::new(addr).try_bind().await {
        Ok(acceptor) => acceptor,
        Err(err) => {
            log::error!("Failed to bind HTTP server on {addr}: {err:?}");
            return;
        }
    };

    log::info!("HTTP server listening on {addr}");
    Server::new(acceptor).serve(router).await;
}
