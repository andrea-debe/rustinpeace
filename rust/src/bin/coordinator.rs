use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant; // NUEVO: Para la medición de rendimiento
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use mandelbrot_distribuido::mandelbrot::calculator_server::{Calculator, CalculatorServer};
use mandelbrot_distribuido::mandelbrot::{Empty, ResultPayload, Task};

#[derive(Debug)]
struct CoordinatorState {
    pending_tasks: Vec<u32>,
    width: u32,
    height: u32,
    max_iterations: u32,
    completed_tasks: u32,
    start_time: Instant,        
    image_buffer: Vec<u8>,
    active_workers: HashSet<String>,
}

#[derive(Debug)]
struct MyCoordinator {
    state: Arc<Mutex<CoordinatorState>>,
}

#[tonic::async_trait]
impl Calculator for MyCoordinator {
    async fn get_task(&self, _request: Request<Empty>) -> Result<Response<Task>, Status> {
        let mut state = self.state.lock().await;

        if let Some(row) = state.pending_tasks.pop() {
            let task = Task {
                task_id: row,
                row,
                width: state.width,
                height: state.height,
                max_iterations: state.max_iterations,
            };
            Ok(Response::new(task))
        } else {
            Err(Status::not_found("No hay más tareas disponibles."))
        }
    }

    async fn submit_result(
        &self,
        request: Request<ResultPayload>,
    ) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner();
        let mut state = self.state.lock().await;
	
	state.active_workers.insert(payload.worker_id.clone());

        // 1. Calculamos en qué posición del lienzo maestro va esta fila
        let start_idx = (payload.task_id * state.width) as usize;
        let end_idx = start_idx + (state.width as usize);

        // 2. Copiamos los bytes crudos directamente al lienzo maestro
        if payload.data.len() == state.width as usize {
            state.image_buffer[start_idx..end_idx].copy_from_slice(&payload.data);
        }

        state.completed_tasks += 1;
        
        // Imprimimos progreso cada 100 filas para no saturar la terminal
        if state.completed_tasks % 100 == 0 || state.completed_tasks == state.height {
            println!(
                "Progreso: {}/{} | Workers activos detectados: {}", 
                state.completed_tasks, 
                state.height,
                state.active_workers.len() // ¡Muestra el tamaño de la lista!
            );
    	}

        // 3. ¿Terminamos? Detener reloj y guardar imagen
        if state.completed_tasks == state.height {
            let duration = state.start_time.elapsed(); // Detenemos el cronómetro
            
            println!("\n====================================================");
            println!("¡CÁLCULO DISTRIBUIDO DE MANDELBROT COMPLETADO!");
            println!("Tiempo total de ejecución: {:.2?}", duration); // Requisito de rendimiento
            println!("====================================================\n");

            println!("Ensamblando y guardando mandelbrot.png...");
            
            // Guardamos el arreglo de bytes como una imagen en escala de grises (L8)
            if let Err(e) = image::save_buffer(
                "mandelbrot.png",
                &state.image_buffer,
                state.width,
                state.height,
                image::ColorType::L8,
            ) {
                println!("Error al guardar la imagen: {}", e);
            } else {
                println!("¡Imagen guardada con éxito en la carpeta rust!");
            }
        }

        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3000".parse().unwrap();

    // Aumentamos un poco la resolución para que se vea genial
    let width = 1920;
    let height = 1080;
    
    let mut pending_tasks: Vec<u32> = (0..height).collect();
    pending_tasks.reverse();

    let state = CoordinatorState {
        pending_tasks,
        width,
        height,
        max_iterations: 1000,
        completed_tasks: 0,
        start_time: Instant::now(), // Disparamos el cronómetro
        image_buffer: vec![0; (width * height) as usize], // Creamos el lienzo en blanco (negro)
        active_workers: HashSet::new(),
    };

    let coordinator = MyCoordinator {
        state: Arc::new(Mutex::new(state)),
    };

    println!("Coordinador gRPC iniciado y escuchando en {}", addr);
    println!("Resolución del fractal: {}x{}", width, height);

    Server::builder()
        .add_service(CalculatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
