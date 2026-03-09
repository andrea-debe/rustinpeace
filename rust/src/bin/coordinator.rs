use std::collections::HashMap; // Cambiamos HashSet por HashMap
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use mandelbrot_distribuido::mandelbrot::calculator_server::{Calculator, CalculatorServer};
use mandelbrot_distribuido::mandelbrot::{Empty, ResultPayload, Task};

#[derive(Debug)]
struct CoordinatorState {
    pending_tasks: Vec<u32>,
    in_progress: HashMap<u32, Instant>, // NUEVO: Tareas prestadas y la hora en que se prestaron
    width: u32,
    height: u32,
    max_iterations: u32,
    completed_tasks: u32,
    start_time: Instant,
    image_buffer: Vec<u8>,
    worker_heartbeats: HashMap<String, Instant>, // NUEVO: Registro de "última vez visto" por worker
}

#[derive(Debug)]
struct MyCoordinator {
    state: Arc<Mutex<CoordinatorState>>,
}

#[tonic::async_trait]
impl Calculator for MyCoordinator {
    async fn get_task(&self, _request: Request<Empty>) -> Result<Response<Task>, Status> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        let timeout = Duration::from_secs(15); // Si un worker tarda más de 15s en una fila, lo damos por muerto

        // 1. REVISIÓN DE CAÍDAS: Buscamos si alguna tarea en progreso expiró
        let mut expired_task = None;
        for (&task_id, &start_time) in state.in_progress.iter() {
            if now.duration_since(start_time) > timeout {
                expired_task = Some(task_id);
                break; // Encontramos una, la rescatamos
            }
        }

        // 2. ASIGNACIÓN: Decidimos qué tarea darle al worker
        let task_to_assign = if let Some(id) = expired_task {
            println!(">> ¡Worker caído detectado! Reasignando fila perdida: {}", id);
            Some(id)
        } else {
            // Si no hay tareas expiradas, sacamos una nueva de la cola normal
            state.pending_tasks.pop()
        };

        // 3. ENTREGAMOS LA TAREA
        if let Some(row) = task_to_assign {
            // La registramos como "En progreso" con la hora actual
            state.in_progress.insert(row, Instant::now());
            
            let task = Task {
                task_id: row,
                row,
                width: state.width,
                height: state.height,
                max_iterations: state.max_iterations,
            };
            Ok(Response::new(task))
        } else {
            // Si no hay tareas nuevas ni expiradas, verificamos si ya terminamos todo
            if state.in_progress.is_empty() {
                Err(Status::not_found("¡Fractal completado!"))
            } else {
                // Hay tareas calculándose, pero aún no expiran. Le decimos al worker que espere.
                Err(Status::unavailable("Esperando a que los demás terminen sus tareas..."))
            }
        }
    }

    async fn submit_result(
        &self,
        request: Request<ResultPayload>,
    ) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner();
        let mut state = self.state.lock().await;
        let now = Instant::now();

        // Actualizamos el "Latido de vida" de este worker
        state.worker_heartbeats.insert(payload.worker_id.clone(), now);

        // SOLO procesamos el resultado si la tarea sigue en nuestra lista "En progreso"
        // (Esto evita que si un worker "revive" tarde, arruine el contador sumando doble)
        if state.in_progress.remove(&payload.task_id).is_some() {
            let start_idx = (payload.task_id * state.width) as usize;
            let end_idx = start_idx + (state.width as usize);

            if payload.data.len() == state.width as usize {
                state.image_buffer[start_idx..end_idx].copy_from_slice(&payload.data);
            }

            state.completed_tasks += 1;
            
            // LIMPIEZA DE WORKERS: Borramos del contador a los que llevan más de 15s sin reportarse
            let heartbeat_timeout = Duration::from_secs(15);
            state.worker_heartbeats.retain(|_, &mut last_seen| now.duration_since(last_seen) < heartbeat_timeout);

            if state.completed_tasks % 100 == 0 || state.completed_tasks == state.height {
                println!(
                    "Progreso: {}/{} | Workers activos reales: {}", 
                    state.completed_tasks, 
                    state.height,
                    state.worker_heartbeats.len() // Ahora sí, el número será 100% exacto
                );
            }

            if state.completed_tasks == state.height {
                let duration = state.start_time.elapsed();
                
                println!("\n====================================================");
                println!("¡CÁLCULO DISTRIBUIDO COMPLETADO!");
                println!("Tiempo total de ejecución: {:.2?}", duration);
                println!("====================================================\n");

                if let Err(e) = image::save_buffer(
                    "mandelbrot.png",
                    &state.image_buffer,
                    state.width,
                    state.height,
                    image::ColorType::L8,
                ) {
                    println!("Error al guardar la imagen: {}", e);
                } else {
                    println!("¡Imagen guardada con éxito!");
                }
            }
        }

        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3000".parse().unwrap();
    let width = 1920;
    let height = 1080;
    
    let mut pending_tasks: Vec<u32> = (0..height).collect();
    pending_tasks.reverse();

    let state = CoordinatorState {
        pending_tasks,
        in_progress: HashMap::new(), // Inicializamos vacío
        width,
        height,
        max_iterations: 1000,
        completed_tasks: 0,
        start_time: Instant::now(),
        image_buffer: vec![0; (width * height) as usize],
        worker_heartbeats: HashMap::new(), // Inicializamos vacío
    };

    let coordinator = MyCoordinator {
        state: Arc::new(Mutex::new(state)),
    };

    println!("Coordinador escuchando en {}", addr);

    Server::builder()
        .add_service(CalculatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
