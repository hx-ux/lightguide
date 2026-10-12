use axum::{Json, Router, extract::Query, routing::get};
use backend::targetdevice::TargetDevice;
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct NoteScaleRequest {
    note: String,
    scale: String,
}

#[derive(Serialize)]
struct GenericResponse {
    status: String,
    message: String,
}

async fn note_scale(Query(payload): Query<NoteScaleRequest>) -> Json<GenericResponse> {
    println!("Received note: {}, scale: {}", payload.note, payload.scale);

    Json(GenericResponse {
        status: "success".to_string(),
        message: format!("Set note to {}, scale to {}", payload.note, payload.scale),
    })
}

async fn notes_off() -> Json<GenericResponse> {
    println!("Received notes off command");

    // Add logic to forward the "notes off" call to the backend

    Json(GenericResponse {
        status: "success".to_string(),
        message: "All notes turned off".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let mut tg = TargetDevice::default();
    // sudo chmod a+rw /dev/hidraw10
    // sudo chmod 600 /dev/hidraw10
    match tg.open_hid_device() {
        Ok(device) => {
            tg.notes_off();

            let app = Router::new().route("/note-scale", get(note_scale));
            // .route("/notes-off", get(notes_off))

            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            println!(
                "REST API server listening on {}",
                listener.local_addr().unwrap()
            );
            axum::serve(listener, app).await.unwrap();
        }
        Err(err) => {
            let err_result = format!("Cannot open device {}", err.to_string());
            println!("Cannot open device");
        }
    }
}
