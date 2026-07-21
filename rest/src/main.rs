use rocket::{get, launch, routes, Build, Rocket};

// http://localhost:8000/status
#[get("/")]
fn device() -> &'static str {
    "Device Info"
}

// http://localhost:8000/keyboard/D
#[get("/<name>")]
fn note(name: &str) -> String {
    format!("Note {}", name)
}

#[get("/<name>")]
fn scale(name: &str) -> String {
    format!("scale {}", name)
}

#[get("/")]
fn home() -> String {
    format!("Home")
}

#[launch]
fn rocket() -> Rocket<Build> {
    // start bg device

    rocket::build()
        .mount("/", routes![home])
        .mount("/status", routes![device])
        .mount("/root_note", routes![note])
        .mount("/scale", routes![scale])
}
