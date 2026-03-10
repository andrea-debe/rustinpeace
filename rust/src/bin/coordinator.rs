//importacion de librerias
use std::collections::HashMap; //Diccionario
use std::sync::Arc; //Atomic Reference Counted pointer (compartir datos inmutables entre multiples hilos)
use std::time::{Duration, Instant}; //Tiempo
use tokio::sync::Mutex; //Bloque mutuo
use tonic::{transport::Server, Request, Response, Status}; //Elementos gRPC

//Importacionn de .proto
use mandelbrot_distribuido::mandelbrot::calculator_server::{Calculator, CalculatorServer}; //trait del servicio gRPC
use mandelbrot_distribuido::mandelbrot::{Empty, ResultPayload, Task}; //Mensajes gRPC

//Estructura del estado global
#[derive(Debug)]
struct CoordinatorState {
    pending_tasks: Vec<u32>, //Cola de tareas pendientes
    in_progress: HashMap<u32, Instant>, //Tareas actualmente en calculo (task_id -> tiempo en que se asigno)
    width: u32, //Ancho de imagen
    height: u32,//Alto de imagen
    max_iterations: u32, //Numero maximo de iteraciones para calcular Mandelbrot.
    completed_tasks: u32, //Numero de filas ya calculadas
    start_time: Instant, //Momento en el que comenzo el calculo
    image_buffer: Vec<u8>, //Arreglo donde se guardan todos los pixeles de la imagen final.
    worker_heartbeats: HashMap<String, Instant>, //Control de vida de worker (worker_id -> ultima vez que reporto)
}

//Estructura de Coordinador
#[derive(Debug)]
struct MyCoordinator {
    state: Arc<Mutex<CoordinatorState>>, //Estado compartido que permite acceso concurrente y seguridad en multiples request
}

//Implementacion del servicio gRPC
#[tonic::async_trait]
impl Calculator for MyCoordinator {
    async fn get_task(&self, _request: Request<Empty>) -> Result<Response<Task>, Status> { //Funcion get_task de .proto (worker solicita trabajo)
        let mut state = self.state.lock().await; //Bloqueo de estado para su modificacion
        let now = Instant::now(); //Obtener tiempo actual
        let timeout = Duration::from_secs(15); //Tiempo maximo de respueta de worker

        //Deteccion de workers caidos
        let mut expired_task = None; //Variable donde se guarda una tarea perdida
        for (&task_id, &start_time) in state.in_progress.iter() { //Iteracion sobre tareas en progreso
            if now.duration_since(start_time) > timeout { //Si se supera el tiempo maximo de respuesta
                expired_task = Some(task_id); //Se marca la tarea como expirada y se guarda en variable
                break; 
            }
        }

        //Asignacion de tarea a worker
        let task_to_assign = if let Some(id) = expired_task { //Si encontramos una tarea expirada
            println!(">> ¡Worker caído detectado! Reasignando fila perdida: {}", id);
            Some(id) //Reasignamos tarea
        } else {
            // Si no hay tareas expiradas, sacamos una nueva de la cola normal
            state.pending_tasks.pop()
        };

        //Entrega de tarea
        if let Some(row) = task_to_assign { //Si existe una tarea disponible
            state.in_progress.insert(row, Instant::now()); // La registramos como "En progreso" con la hora actual
            //Creamos la estructura de Task
            let task = Task {
                task_id: row,
                row,
                width: state.width,
                height: state.height,
                max_iterations: state.max_iterations,
            };
            Ok(Response::new(task)) //Enviamos respuesta gRPC
        } else {
            // Si no hay tareas nuevas ni expiradas, verificamos si ya terminamos todo
            if state.in_progress.is_empty() {
                Err(Status::not_found("¡Fractal completado!"))
            } else {
                // Hay tareas calculandose, pero aun no expiran. Le decimos al worker que espere.
                Err(Status::unavailable("Esperando a que los demás terminen sus tareas..."))
            }
        }
    }
    
    //Funcion para que el Worker entregue resultados
    async fn submit_result(
        &self,
        request: Request<ResultPayload>,
    ) -> Result<Response<Empty>, Status> {
        let payload = request.into_inner(); //Extrae el mensaje gRPC y obtiene Payload
        let mut state = self.state.lock().await; //Bloquea estado
        let now = Instant::now(); //Tiempo actual

        state.worker_heartbeats.insert(payload.worker_id.clone(), now); //Actualiza el ultimo contacto del worker

        // Validamos tarea - SOLO procesamos el resultado si la tarea sigue en nuestra lista "En progreso"
        if state.in_progress.remove(&payload.task_id).is_some() {
            let start_idx = (payload.task_id * state.width) as usize; //Inicio de la fila
            let end_idx = start_idx + (state.width as usize); //Fin de la fila

            if payload.data.len() == state.width as usize {
                state.image_buffer[start_idx..end_idx].copy_from_slice(&payload.data); //Copia los pixeles calculados a image_buffer
            }

            state.completed_tasks += 1; //Actualiza el progreso
            
            // Borra del contador a los que llevan mas de 15s sin reportarse
            let heartbeat_timeout = Duration::from_secs(15);
            state.worker_heartbeats.retain(|_, &mut last_seen| now.duration_since(last_seen) < heartbeat_timeout);

            if state.completed_tasks % 100 == 0 || state.completed_tasks == state.height { //Cada 100 filas imprimimos
                println!(
                    "Progreso: {}/{} | Workers activos reales: {}", 
                    state.completed_tasks, 
                    state.height,
                    state.worker_heartbeats.len() 
                );
            }

            if state.completed_tasks == state.height { //Si el progreso coincide con la altura de la imagen, se termino todo
                let duration = state.start_time.elapsed(); //Medida de teimpo transcurrido
                
                println!("\n====================================================");
                println!("¡CÁLCULO DISTRIBUIDO COMPLETADO!");
                println!("Tiempo total de ejecución: {:.2?}", duration);
                println!("====================================================\n");

                if let Err(e) = image::save_buffer( //Guardamos el buffer como imagen png
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

//Funcion main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:3000".parse().unwrap(); //El servidor escuha el puesto 3000
    //Definimos resolucion (dimensiones) de la imagen
    let width = 1920;
    let height = 1080;
    
    let mut pending_tasks: Vec<u32> = (0..height).collect(); //Crea la lista de tareas
    pending_tasks.reverse(); //Se invierte para usar pop

    //Inicializa todo el sistema
    let state = CoordinatorState {
        pending_tasks,
        in_progress: HashMap::new(), // Inicializamos vacío
        width,
        height,
        max_iterations: 1000,
        completed_tasks: 0,
        start_time: Instant::now(),
        image_buffer: vec![0; (width * height) as usize], //Inicializamos vector lleno de ceros
        worker_heartbeats: HashMap::new(), // Inicializamos vacío
    };

    //Estado compartido entre requests
    let coordinator = MyCoordinator {
        state: Arc::new(Mutex::new(state)),
    };

    println!("Coordinador escuchando en {}", addr);

    Server::builder() //Crea el servidor
        .add_service(CalculatorServer::new(coordinator)) //Resgitra el servicio gRPC
        .serve(addr) //Inicia el servidor
        .await?;

    Ok(())
}
