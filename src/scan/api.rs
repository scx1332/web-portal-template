use crate::db::model::UserDbObj;
use crate::db::ops::transaction::{get_all_scans, get_blocks, get_scan};
use crate::ServerData;
use actix_session::Session;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Scope};
use lazy_static::lazy_static;

lazy_static! {
    static ref IGNORE_SCAN_API_LOGIN: bool = {
        let val = std::env::var("IGNORE_SCAN_API_LOGIN").unwrap_or_default();
        val == "1" || val.to_lowercase() == "true"
    };
}

//macro login check
// Define the macro for login check
macro_rules! login_check {
    ($session:expr) => {
        if *IGNORE_SCAN_API_LOGIN {
            // Ignore login check
        } else if let Some(_usr_db_obj) = $session.get::<UserDbObj>("user").unwrap_or(None) {
            // User is logged in, so just proceed.
        } else {
            // User is not logged in, return an unauthorized error
            return HttpResponse::Unauthorized().body("Not logged in");
        }
    };
}

async fn web_get_scan_info(
    data: Data<Box<ServerData>>,
    address: web::Path<String>,
    session: Session,
) -> HttpResponse {
    login_check!(session);

    let db = data.db_connection.lock().await;

    match get_scan(&db, &address).await {
        Ok(scan_info) => HttpResponse::Ok().json(scan_info),
        Err(e) => {
            log::error!("Error getting scan info: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn web_get_all_scans(data: Data<Box<ServerData>>, session: Session) -> HttpResponse {
    login_check!(session);

    let db = data.db_connection.lock().await;

    match get_all_scans(&db).await {
        Ok(scans) => HttpResponse::Ok().json(scans),
        Err(e) => {
            log::error!("Error getting scan info: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn web_get_blocks(
    data: Data<Box<ServerData>>,
    address: web::Path<String>,
    session: Session,
) -> HttpResponse {
    login_check!(session);

    let db = data.db_connection.lock().await;

    match get_blocks(&db, &address).await {
        Ok(blocks) => HttpResponse::Ok().json(blocks),
        Err(e) => {
            log::error!("Error getting scan info: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn get_scan_scope() -> Scope {
    let api_scope = Scope::new("/scan");

    api_scope
        .route("{address}/info", web::get().to(web_get_scan_info))
        .route("{address}/blocks", web::get().to(web_get_blocks))
        .route("all", web::get().to(web_get_all_scans))
}
