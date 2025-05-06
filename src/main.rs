use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rusqlite::{Connection, Result as SqliteResult};
use chrono::naive::NaiveDate;
use actix_files::Files;

#[derive(Debug)]
struct Workout {
    id: i32,
    date: NaiveDate,
    duration: i32 // In minutes
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut _conn = Connection::open("workout.db").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    _conn.execute(
        "CREATE TABLE IF NOT EXISTS workouts (
            id INTEGER PRIMARY KEY,
            date DATETIME NOT NULL,
            duration INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    HttpServer::new(|| {
        App::new()
            .service(Files::new("/", "./static").index_file("index.html"))
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}