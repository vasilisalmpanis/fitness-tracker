use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use std::sync::Mutex;

#[derive(Deserialize, Debug)]
struct WorkoutInput {
    date: String,
    value: i32,
}

#[derive(Serialize, Debug)]
struct Workout {
    id: i32,
    date: String,
    value: i32,
}

struct AppState {
    db: Mutex<Connection>,
}

// #[get("/")]
// async fn hello() -> impl Responder {
//     HttpResponse::Ok().body("Hello world!")
// }

#[post("/add")]
async fn add(
    workout_input: web::Json<WorkoutInput>,
    data: web::Data<AppState>
) -> impl Responder {
    println!("Received workout: {:?}", workout_input);

    if workout_input.value < 0 || workout_input.value > 120 {
        return HttpResponse::BadRequest().body("duration");
    }

    let _parsed_date = match NaiveDate::parse_from_str(&workout_input.date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(e) => {
            println!("Date parsing error: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": format!("Invalid date format: {}", e)
            }));
        }
    };

    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database connection error"
            }));
        }
    };

    let existing_workout = db.query_row(
        "SELECT id, date, value FROM workouts WHERE date = ?1",
        params![workout_input.date],
        |row| {
            Ok(Workout {
                id: row.get(0)?,
                date: row.get(1)?,
                value: row.get(2)?,
            })
        }
    );

    match existing_workout {
        Ok(mut existing) => {
            println!("Workout exists for date {}, updating...", workout_input.date);

            match db.execute(
                "UPDATE workouts SET value = ?1 WHERE date = ?2",
                params![workout_input.value, workout_input.date],
            ) {
                Ok(_) => {
                    existing.value = workout_input.value;
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "updated",
                        "message": "Workout updated successfully",
                        "workout": existing
                    }))
                }
                Err(e) => {
                    println!("Database update error: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": "Failed to update workout"
                    }))
                }
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // No workout exists - create new one
            println!("No workout exists for date {}, creating new...", workout_input.date);

            match db.execute(
                "INSERT INTO workouts (date, value) VALUES (?1, ?2)",
                params![workout_input.date, workout_input.value],
            ) {
                Ok(_) => {
                    // Get the ID of the newly inserted workout
                    let new_id = db.last_insert_rowid() as i32;
                    let new_workout = Workout {
                        id: new_id,
                        date: workout_input.date.clone(),
                        value: workout_input.value,
                    };

                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "created",
                        "message": "Workout created successfully",
                        "workout": new_workout
                    }))
                }
                Err(e) => {
                    println!("Database insert error: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "status": "error",
                        "message": "Failed to create workout"
                    }))
                }
            }
        }
        Err(e) => {
            // Other database error
            println!("Database query error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database query failed"
            }))
        }
    }
}

#[get("/workouts")]
async fn workouts(data: web::Data<AppState>) -> impl Responder {
    println!("Fetching all workouts from database");

    // Get database connection
    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database connection error"
            }));
        }
    };

    // Query all workouts
    let mut stmt = match db.prepare("SELECT id, date, value FROM workouts ORDER BY date DESC") {
        Ok(stmt) => stmt,
        Err(e) => {
            println!("Failed to prepare statement: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Database query preparation failed"
            }));
        }
    };

    let workout_iter = stmt.query_map([], |row| {
        Ok(Workout {
            id: row.get(0)?,
            date: row.get(1)?,
            value: row.get(2)?,
        })
    });

    match workout_iter {
        Ok(workouts) => {
            let mut workout_list = Vec::new();

            for workout in workouts {
                match workout {
                    Ok(w) => workout_list.push(w),
                    Err(e) => {
                        println!("Error reading workout row: {}", e);
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "status": "error",
                            "message": "Error reading workout data"
                        }));
                    }
                }
            }

            HttpResponse::Ok().json(serde_json::json!({
                // "status": "success",
                "workouts": workout_list,
                // "count": workout_list.len()
            }))
        }
        Err(e) => {
            println!("Database query error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "Failed to fetch workouts"
            }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Initialize database
    let conn = Connection::open("workout.db")
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS workouts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL UNIQUE,
            value INTEGER NOT NULL
        )",
        [],
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Wrap connection in app state
    let app_state = web::Data::new(AppState {
        db: Mutex::new(conn),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            // .service(hello)
            .service(add)
            .service(workouts)
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
