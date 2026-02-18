use axum::{
    routing::{get, post},
    Json, Router,
};
use mandelbrot_distribuido::{ResultPayload, Task};
use std::net::SocketAddr;

async fn get_task() -> Json<Task> {
    let dummy_task = Task {
        task_id: 1,
        instruction: "Calcula una tarea dummy de prueba".to_string(),
    };
    println!("Enviando tarea: {:?}", dummy_task);
    Json(dummy_task)
}

async fn submit_result(Json(payload): Json<ResultPayload>) -> &'static str {
    println!("Resultado recibido del worker '{}': {:?}", payload.worker_id, payload.data);
    "Resultado aceptado"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/task", get(get_task))
        .route("/result", post(submit_result));

    // Escucha en el puerto 3000 en todas las interfaces de red
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Coordinador iniciado. Escuchando peticiones en http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}