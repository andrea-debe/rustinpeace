use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

// Importamos el código que Protobuf generó mágicamente por nosotros
use mandelbrot_distribuido::mandelbrot::calculator_server::{Calculator, CalculatorServer};
use mandelbrot_distribuido::mandelbrot::{Empty, ResultPayload, Task};

// 1. Definimos el "Estado" de nuestro servidor
#[derive(Debug)]
struct CoordinatorState {
    pending_tasks: Vec<u32>, // Una cola con los números de las filas a calcular
    width: u32,              // Ancho de la imagen final
    height: u32,             // Alto de la imagen final
    max_iterations: u32,     // Calidad del fractal
    completed_tasks: u32,    // Contador para saber cuándo terminamos
}

// 2. Definimos nuestra estructura principal que guardará el estado protegido
#[derive(Debug)]
struct MyCoordinator {
    state: Arc<Mutex<CoordinatorState>>,
}

// 3. Implementamos el trait Calculator (El contrato de gRPC)
#[tonic::async_trait]
impl Calculator for MyCoordinator {
    
    // Función que los workers llaman para pedir trabajo
    async fn get_task(&self, _request: Request<Empty>) -> Result<Response<Task>, Status> {
        // Bloqueamos el estado temporalmente para que ningún otro worker lo toque
        let mut state = self.state.lock().await;

        // Sacamos la última fila disponible de la cola
        if let Some(row) = state.pending_tasks.pop() {
            println!(">> Asignando fila {} a un worker", row);
            
            let task = Task {
                task_id: row, // Usamos la misma fila como ID
                row,
                width: state.width,
                height: state.height,
                max_iterations: state.max_iterations,
            };
            
            Ok(Response::new(task)) // Enviamos la tarea
        } else {
            // Si la cola está vacía, le decimos al worker que ya no hay trabajo
            Err(Status::not_found("No hay más tareas disponibles. ¡Fractal completado!"))
        }
    }

    // Función que los workers llaman para entregar resultados
    async fn submit_result(
        &self,
        request: Request<ResultPayload>,
    ) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner();
        let mut state = self.state.lock().await;

        state.completed_tasks += 1;
        println!(
            "<< Resultado de fila {} recibido del {}. Progreso: {}/{}",
            payload.task_id, payload.worker_id, state.completed_tasks, state.height
        );

        // NOTA: Aquí es donde en el futuro tomaremos `payload.data` (los bytes) 
        // y los pintaremos en nuestra imagen PNG.

        if state.completed_tasks == state.height {
            println!("\n¡CÁLCULO DISTRIBUIDO DE MANDELBROT COMPLETADO EXITOSAMENTE!");
        }

        Ok(Response::new(Empty {}))
    }
}

// 4. El punto de entrada principal
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Escuchamos en todas las interfaces, puerto 3000
    let addr = "0.0.0.0:3000".parse().unwrap();

    // Configuramos nuestro fractal (Ej. 800x800 píxeles)
    let height = 800;
    let width = 800;
    
    // Llenamos la cola con las filas de la 0 a la 799
    let mut pending_tasks: Vec<u32> = (0..height).collect();
    // Le damos la vuelta para que el método .pop() empiece sacando la fila 0
    pending_tasks.reverse();

    let state = CoordinatorState {
        pending_tasks,
        width,
        height,
        max_iterations: 1000,
        completed_tasks: 0,
    };

    let coordinator = MyCoordinator {
        state: Arc::new(Mutex::new(state)),
    };

    println!("Coordinador gRPC iniciado y escuchando en {}", addr);
    println!("Total de tareas encoladas: {}", height);

    // Levantamos el servidor de alto rendimiento
    Server::builder()
        .add_service(CalculatorServer::new(coordinator))
        .serve(addr)
        .await?;

    Ok(())
}
