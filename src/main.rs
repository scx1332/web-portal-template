mod api;
mod cookie;
mod db;
mod error;
mod scan;
mod update;

use crate::api::user;
use crate::api::user::{UserSessions, WEB_PORTAL_DOMAIN};
use crate::cookie::load_key_or_create;
use crate::db::connection::create_sqlite_connection;
use crate::scan::api::get_scan_scope;
use actix_multipart::form::MultipartFormConfig;
use actix_multipart::MultipartError;
use actix_session::config::CookieContentSecurity;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::SameSite;
use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Responder, Scope,
};
use awc::http::StatusCode;
use awc::Client;
use clap::{Parser, Subcommand};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ServerData {
    pub db_connection: Arc<Mutex<SqlitePool>>,
    pub open_sessions: Arc<Mutex<HashMap<String, UserSessions>>>,
}

#[cfg(feature = "dashboard")]
#[derive(rust_embed::RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;

pub async fn redirect_to_dashboard() -> impl Responder {
    {
        let target = "/dashboard/";
        log::debug!("Redirecting to endpoint: {target}");
        HttpResponse::Ok()
            .status(actix_web::http::StatusCode::PERMANENT_REDIRECT)
            .append_header((actix_web::http::header::LOCATION, target))
            .finish()
    }
}

#[allow(dead_code)]
async fn proxy(
    path: web::Path<String>,
    client: web::Data<Client>,
    request: HttpRequest,
) -> HttpResponse {
    log::info!("Proxying request to: {path}");
    let url = format!("http://localhost:5173/dashboard/{path}");

    // here we use `IntoHttpResponse` to return the request to
    // duckduckgo back to the client that called this endpoint

    let mut new_request = client.request(request.method().clone(), url);
    for (header_name, header_value) in request.headers() {
        new_request = new_request.insert_header((header_name.clone(), header_value.clone()));
    }
    match new_request.send().await {
        Ok(resp) => {
            log::info!("Response: {}", resp.status());
            let mut response = HttpResponse::build(resp.status());

            resp.headers().into_iter().for_each(|(k, v)| {
                response.insert_header((k, v));
            });

            response.streaming(resp)
        }
        Err(e) => {
            log::error!("Error: {e}");
            HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(format!("Error: {e}"))
        }
    }
}

#[allow(clippy::needless_return)]
#[allow(unreachable_code)]
pub async fn dashboard_serve(
    path: web::Path<String>,
    _client: web::Data<Client>,
    _request: HttpRequest,
) -> HttpResponse {
    #[cfg(feature = "dashboard")]
    {
        let mut path = path.as_str();
        let mut content = Asset::get(path);
        if content.is_none() && !path.contains('.') {
            path = "index.html";
            content = Asset::get(path);
        }
        log::debug!("Serving frontend file: {path}");
        return match content {
            Some(content) => HttpResponse::Ok()
                .content_type(mime_guess::from_path(path).first_or_octet_stream().as_ref())
                .body(content.data.into_owned()),
            None => HttpResponse::NotFound().body("404 Not Found"),
        };
    }
    #[cfg(feature = "proxy")]
    {
        return proxy(path, _client, _request).await;
    }
    #[cfg(all(not(feature = "dashboard"), not(feature = "proxy")))]
    HttpResponse::NotFound().body(format!("404 Not Found: {}", path))
}

use crate::scan::cmd::scan_command;

/// Enum that defines the available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Scan blockchain
    Scan {
        #[clap(flatten)]
        scan: scan::cmd::ScanCommand,
    },
    /// Start web server
    Server {
        #[arg(long, default_value = "localhost:80")]
        addr: String,

        #[arg(long)]
        threads: Option<usize>,
    },
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,

    #[arg(long, default_value = "web-portal.sqlite")]
    db: String,
}

fn handle_multipart_error(err: MultipartError, _req: &HttpRequest) -> actix_web::Error {
    log::error!("Multipart error: {}", err);
    actix_web::Error::from(err)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG").unwrap_or("info".to_string()),
    );
    env_logger::init();

    let args = Cli::parse();

    let conn = create_sqlite_connection(Some(&PathBuf::from(args.db)), None, false, true)
        .await
        .unwrap();

    let secret_key = load_key_or_create("web-portal-cookie.key");

    match args.cmd {
        Commands::Scan { scan } => scan_command(conn, scan).await.map_err(|e| {
            log::error!("Error: {e}");
            std::io::Error::new(std::io::ErrorKind::Other, format!("Error: {e}"))
        }),
        Commands::Server { addr, threads } => {
            HttpServer::new(move || {
                let cors = actix_cors::Cors::permissive();

                let server_data = web::Data::new(Box::new(ServerData {
                    db_connection: Arc::new(Mutex::new(conn.clone())),
                    open_sessions: Arc::new(Default::default()),
                }));
                let client = web::Data::new(Client::new());
                let session_middleware =
                    SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                        .cookie_secure(true)
                        .cookie_content_security(CookieContentSecurity::Private)
                        .cookie_same_site(SameSite::Strict)
                        .cookie_domain(Some(WEB_PORTAL_DOMAIN.to_string()))
                        .cookie_name("web-portal-session".to_string())
                        .build();

                let api_scope = Scope::new("/api")
                    .service(get_scan_scope())
                    .route("/login", web::post().to(user::handle_login))
                    .route("/session/check", web::get().to(user::handle_session_check))
                    .route("/is_login", web::get().to(user::handle_is_login))
                    .route("/is_login", web::post().to(user::handle_is_login))
                    .route("/logout", web::post().to(user::handle_logout))
                    .route("/change_pass", web::post().to(user::handle_password_change))
                    .route("/greet", web::get().to(user::handle_greet));

                App::new()
                    .wrap(session_middleware)
                    .wrap(cors)
                    .app_data(server_data)
                    .app_data(client)
                    .app_data(
                        MultipartFormConfig::default()
                            .total_limit(10 * 1024 * 1024 * 1024) // 10 GB
                            .memory_limit(10 * 1024 * 1024) // 10 MB
                            .error_handler(handle_multipart_error),
                    )
                    .route("/", web::get().to(redirect_to_dashboard))
                    .route("/dashboard", web::get().to(redirect_to_dashboard))
                    .route("/dashboard/{_:.*}", web::get().to(dashboard_serve))
                    .route("/service/update", web::post().to(update::push_update))
                    .service(api_scope)
            })
            .workers(threads.unwrap_or(std::thread::available_parallelism().unwrap().into()))
            .bind(addr)?
            .run()
            .await
        }
    }
}
