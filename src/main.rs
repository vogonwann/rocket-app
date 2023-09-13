#[macro_use] extern crate rocket;

mod auth;
mod schema;
mod models;
mod repositories;

use diesel::result::Error::NotFound;
use rocket::{serde::json::{Value, json, Json}, response::status::{self, Custom}, http::Status, fairing::AdHoc};
use rocket_sync_db_pools::database;

use auth::BasicAuth;
use models::{Rustacean, NewRustacean};
use schema::rustaceans;

use crate::repositories::RustaceanRepository;

#[database("sqlite")]
struct DbConn(diesel::SqliteConnection); 

#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(|c|  {
        RustaceanRepository::find_all(c, 100)
            .map(|rustaceans| json!(rustaceans))
            .map_err(|err| 
                match err {
                    NotFound => Custom(Status::NotFound, json!({"status": "error", "reason": "Resource was not found."})),
                    _ => Custom(Status::InternalServerError, json!({"status": "error", "reason": err.to_string()}))
                }
            )
    }).await
}

#[get("/rustaceans/<id>")]
async fn view_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::find(c, id)
            .map(|rustacean| json!(rustacean))
            .map_err(|err| 
                Custom(Status::InternalServerError, json!({"status": "error", "reason": err.to_string()})))
    }).await
}

#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustacean(_auth: BasicAuth, db:DbConn, new_rustacean: Json<NewRustacean>) -> Result<Value, Custom<Value>> {
    db.run(|c| {
        RustaceanRepository::create(c, new_rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|err| 
                Custom(Status::InternalServerError, json!({"status": "error", "reason": err.to_string()})))
    }).await
}

#[put("/rustaceans/<id>", format = "json", data = "<rustacean>")]
async fn update_rustacean(id: i32, _auth: BasicAuth, db:DbConn, rustacean: Json<Rustacean>) -> Result<Value, Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::save(c, id, rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|err| 
                Custom(Status::InternalServerError, json!({"status": "error", "reason": err.to_string()})))
    }).await
}

#[delete("/rustaceans/<id>")]
async fn delete_rustacean(id: i32, _auth: BasicAuth, db:DbConn) -> Result<status::NoContent, Custom<Value>> {
   db.run(move |c| {
        RustaceanRepository::delete(c, id)
            .map(|_| status::NoContent)
            .map_err(|err| 
                Custom(Status::InternalServerError, json!({"status": "error", "reason": err.to_string()})))
    }).await
}

#[catch(404)]
fn not_found() -> Value {
    json!({"status": "error", "reason": "Resource was not found."})
}

#[catch(401)]
fn unauthorized() -> Value {
    json!({"status": "error", "reason": "Unauthorized."})
}

#[catch(422)]
fn unprocessable_entity() -> Value {
    json!({"status": "error", "reason": "Unprocessable entity."})
}

async fn run_db_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    DbConn::get_one(&rocket).await.unwrap().run(|c| {
        c.run_pending_migrations(MIGRATIONS).expect("Failed to run database migrations.");
    }).await;

    rocket
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", routes![
            get_rustaceans,
            view_rustacean,
            create_rustacean,
            update_rustacean,
            delete_rustacean
        ])
        .register("/", catchers![
            not_found,
            unauthorized,
            unprocessable_entity
            ])
        .attach(AdHoc::on_ignite("Database Migrations", run_db_migrations))
        .attach(DbConn::fairing())
        .launch()
        .await;
}
