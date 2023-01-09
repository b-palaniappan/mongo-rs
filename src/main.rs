use actix_web::{
    post,
    web::{self, Json},
    App, HttpResponse, HttpServer,
};
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

// -- Controller
#[post("/users")]
async fn add_users(client: web::Data<Client>, new_user: Json<User>) -> HttpResponse {
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(new_user, None).await;
    match result {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
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
