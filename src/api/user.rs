use crate::db::model::UserDbObj;
use crate::db::ops::{get_user, update_user_password};
use crate::ServerData;
use actix_session::Session;
use actix_web::web;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use clap::crate_version;
use lazy_static::lazy_static;
use pbkdf2::pbkdf2_hmac_array;
use rand::Rng;
use rustc_hex::ToHex;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::env;
use std::time::Duration;

lazy_static! {
    static ref ALLOWED_EMAILS: Vec<String> = serde_json::from_str(
        &env::var("ALLOWED_EMAILS").unwrap_or("[\"sieciech.czajka@golem.network\"]".to_string())
    )
    .unwrap();
    pub static ref WEB_PORTAL_DOMAIN: String =
        env::var("WEB_PORTAL_DOMAIN").unwrap_or("localhost".to_string());
    static ref PASS_SALT: String = env::var("PASS_SALT").unwrap_or("LykwVQJAcU".to_string());
}

#[derive(Debug, Clone)]
pub struct UserSessions {
    pub user: UserDbObj,
    pub session_id: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

fn pass_to_hash(password_binary: &[u8]) -> String {
    //decode password
    let salt = PASS_SALT.as_bytes();
    // number of iterations
    let n = 5000;

    let key: String = pbkdf2_hmac_array::<Sha256, 20>(password_binary, salt, n).to_hex();

    if key.len() != 40 {
        panic!("Key length should be 40")
    }
    key
}

pub async fn handle_is_login(session: Session) -> impl Responder {
    if let Some(usr_db_obj) = session.get::<UserDbObj>("user").unwrap_or(None) {
        HttpResponse::Ok().json(usr_db_obj)
    } else {
        HttpResponse::Unauthorized().body("Not logged in")
    }
}

pub async fn handle_logout(session: Session) -> impl Responder {
    if session.get::<UserDbObj>("user").unwrap_or(None).is_none() {
        return HttpResponse::Ok().body("Not logged in");
    }
    session.remove("user");
    HttpResponse::Ok().body("Logged out")
}

pub async fn handle_session_check(session: Session) -> impl Responder {
    if session.get::<String>("check").unwrap_or(None).is_none() {
        session
            .insert("check", uuid::Uuid::new_v4().to_string())
            .unwrap();
    }
    session.get::<String>("check").unwrap().unwrap()
}

pub async fn handle_login(
    data: Data<Box<ServerData>>,
    login: web::Json<LoginData>,
    session: Session,
) -> impl Responder {
    if let Some(user) = session.get::<UserDbObj>("user").unwrap_or(None) {
        return HttpResponse::Ok().json(user);
    }
    // Generate a random number between 300 and 500 (in milliseconds)
    let mut rng = rand::thread_rng();
    let random_duration = rng.gen_range(300..=600);
    tokio::time::sleep(Duration::from_millis(random_duration)).await;

    if !ALLOWED_EMAILS.contains(&login.email) {
        return HttpResponse::Unauthorized().body("This email is not allowed");
    }

    let db_conn = data.db_connection.lock().await;

    let key = pass_to_hash(login.password.as_bytes());

    log::info!("Getting user: {}", login.email);
    let usr = match get_user(&db_conn, &login.email).await {
        Ok(usr) => usr,
        Err(err) => {
            log::error!("Error getting user: {}", err);
            return HttpResponse::Unauthorized().body("Invalid email or password");
        }
    };
    log::info!("Login {} == {}", usr.pass_hash, key);
    if usr.pass_hash == key {
        log::info!("User {} logged in", login.email);
        session.insert("user", &usr).unwrap();

        return HttpResponse::Ok().json(usr);
    }
    HttpResponse::Unauthorized().body("Invalid email or password")
}

pub async fn handle_greet(session: Session) -> impl Responder {
    println!("Session: {:?}", session.status());
    let describe_version = crate_version!();

    HttpResponse::Ok().json(json!({
        "message": "Hello, World!",
        "domain": *WEB_PORTAL_DOMAIN.clone(),
        "version": describe_version,
    }))
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangePassData {
    pub email: String,
    pub old_password: String,
    pub new_password: String,
}

pub async fn handle_password_change(
    data: Data<Box<ServerData>>,
    change_pass: web::Json<ChangePassData>,
    session: Session,
) -> impl Responder {
    if session.get::<String>("user").unwrap_or(None).is_none() {
        return HttpResponse::Unauthorized().body("Not logged in");
    }

    // Simulate a small delay for security reasons (to prevent timing attacks)
    let mut rng = rand::thread_rng();
    let random_duration = rng.gen_range(300..=600);
    tokio::time::sleep(Duration::from_millis(random_duration)).await;

    let db_conn = data.db_connection.lock().await;

    // Hash the old password
    let old_password_hash = pass_to_hash(change_pass.old_password.as_bytes());

    // Fetch the user from the database using the provided email
    log::info!("Fetching user: {}", change_pass.email);
    let usr = match get_user(&db_conn, &change_pass.email).await {
        Ok(usr) => usr,
        Err(err) => {
            log::error!("Error getting user: {}", err);
            return HttpResponse::Unauthorized().body("Invalid email or password");
        }
    };

    // Check if the provided old password matches the stored password hash
    log::info!("Checking old password hash for user: {}", change_pass.email);
    if usr.pass_hash != old_password_hash {
        return HttpResponse::Unauthorized().body("Invalid old password");
    }

    // Hash the new password
    let new_password_hash = pass_to_hash(change_pass.new_password.as_bytes());

    // Update the user's password in the database
    log::info!("Updating password for user: {}", change_pass.email);
    match update_user_password(&db_conn, &change_pass.email, &new_password_hash).await {
        Ok(_) => {
            log::info!(
                "Password successfully updated for user: {}",
                change_pass.email
            );
            HttpResponse::Ok().body("Password changed successfully")
        }
        Err(err) => {
            log::error!(
                "Error updating password for user: {}: {}",
                change_pass.email,
                err
            );
            HttpResponse::InternalServerError().body("Failed to change password")
        }
    }
}
