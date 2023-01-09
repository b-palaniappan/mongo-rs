use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    post,
    web::{self, Json},
    App, HttpResponse, HttpServer,
};
use chrono::{SecondsFormat, Utc};
use derive_more::{Display, Error};
use mongodb::Client;
use serde::{Deserialize, Serialize};

// Constant setup...
const DB_NAME: &str = "mo_app";
const COLL_NAME: &str = "users";

// Data structure MongoDB collection.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: Option<String>,
    first_name: String,
    last_name: String,
    email: String,
}

// Data structure for API Error.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub status_code: u16,
    pub time: String,
    pub message: String,
    pub debug_message: Option<String>,
}

// -- Error handing.
#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

// -- Controllers
#[post("/users")]
async fn add_users(client: web::Data<Client>, new_user: Json<User>) -> HttpResponse {
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(new_user, None).await;
    match result {
        Ok(user) => HttpResponse::Created().json(user.inserted_id),
        Err(_) => HttpResponse::InternalServerError().json(ApiError {
            status_code: 500,
            time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
            message: "Internal server error".to_owned(),
            debug_message: Some("Internal server error. Please try again later.".to_owned()),
        }),
    }
}

// -- Server setup and start.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configure MongoDB connection and client
    let uri = "mongodb://localhost:27017";
    let client = Client::with_uri_str(uri)
        .await
        .expect("Error connecting to Mongo DB.");

    // Initialize server, set app_data and start server.
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(add_users)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
