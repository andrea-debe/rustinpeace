//Importacion de librerias
use std::time::Duration; //Tiempo
use tonic::Request; //Estrcutura de peticion de tonic

// Importaciones de .proto
use mandelbrot_distribuido::mandelbrot::calculator_client::CalculatorClient; //Cliente gRPC
use mandelbrot_distribuido::mandelbrot::{Empty, ResultPayload}; //Mensajes gRPC

// Motor matematico
// Calculo de una fila completa de mandelbrot
fn calculate_mandelbrot_row(row: u32, width: u32, height: u32, max_iterations: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(width as usize); //Crea buffer de pixeles

    // Límites matemáticos del fractal en el plano cartesiano
    let x_min = -2.0;
    let x_max = 1.0;
    let y_min = -1.5;
    let y_max = 1.5;

    // Convierte fila de imagen en una coordenada Y del plano cartesiano
    let y0 = y_min + (row as f64 / height as f64) * (y_max - y_min);

    for col in 0..width { //Itera sobre todas las columnas de la fila
        let x0 = x_min + (col as f64 / width as f64) * (x_max - x_min); // Convierte el pixel actual en una coordenada X del plano cartesiano

        //Incializa variables de fractal
        let mut x = 0.0;
        let mut y = 0.0;
        let mut iteration = 0;

        // Formula: Z = Z^2 + C - Criterio de escape
        while x * x + y * y <= 4.0 && iteration < max_iterations { //Si la magnitud supera 4 = el punto escapa al infinito y no pertenece
            //Implementacion de la ecuacion
            let x_temp = x * x - y * y + x0;
            y = 2.0 * x * y + y0;
            x = x_temp;
            iteration += 1; //Conteo de iteraciones
        }

        // Convierte las iteraciones en un valor de "color" (un byte de 0 a 255)
        let color_value = (iteration % 256) as u8;
        pixels.push(color_value); //Guarda el pixel en el vector
    }

    // Regresa el arreglo de colores crudos (Bytes) de la fila
    pixels
}

//Funcion main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let coordinator_url = "http://10.0.0.2:3000"; //Apunta a la direccion del coordinador
    //let coordinator_url = "http://127.0.0.1:3000"; //Apunta a la direccion del coordinador (local)
    
    let unique_time = std::time::SystemTime::now() //Obtiene el tiempo actual
        .duration_since(std::time::UNIX_EPOCH) //Convierte
        .unwrap()
        .subsec_nanos(); //Obtiene nanosegundos
        
    let my_worker_id = format!("worker-{}", unique_time); //Crea ID unico de worker con el tiempo (ej. worker-83726481)

    //Mensajes de inicio
    println!("Iniciando worker: {}", my_worker_id);
    println!("Buscando al coordinador en {}...", coordinator_url);

    // Un bucle infinito intentando conectar.
    let mut client = loop {
        match CalculatorClient::connect(coordinator_url).await { //Intento de conexion
            //Si conecta correctamente rompe bucle
            Ok(c) => {
                println!("¡Conexión gRPC establecida exitosamente!\n");
                break c;
            }
            //Si falla espera 3 seg. y vuelve a intentar
            Err(_) => {
                println!("Esperando al coordinador... Reintentando en 3s...");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    };

    //Bucle principal de trabajo
    loop {
        let request = Request::new(Empty {}); //Crea una peticion de tarea vacia
        
        match client.get_task(request).await { //Lama remotamente al coordinador
            //Si recibe tarea
            Ok(response) => { //El coordinador asigno tarea
                let task = response.into_inner(); //Extrae Tarea
                 println!(">> Tarea Recibida: Calcular fila {}", task.row);

                //ejecuta calculo de fractal
                let pixels_data = calculate_mandelbrot_row(
                    task.row,
                    task.width,
                    task.height,
                    task.max_iterations,
                );

                //Empaqueta los bytes binarios
                let result_payload = ResultPayload { //Crea payload de resultado
                    task_id: task.task_id,
                    worker_id: my_worker_id.to_string(),
                    data: pixels_data,
                };

                // Envia el resultado por el tunel gRPC
                let result_req = Request::new(result_payload);
                if let Err(e) = client.submit_result(result_req).await { //Si falla el envio
                    println!("Error enviando resultado al coordinador: {:?}", e);
                } else { //Si funciona
                    println!("<< Fila {} calculada y enviada", task.row);
                }
            }
            Err(status) => { //Si no hay tareas  
                if status.code() == tonic::Code::NotFound { // Si el coordinador responde "NOT_FOUND", significa que ya se calculo toda la imagen
                    println!("\n¡El coordinador informa que el fractal está completo!");
                    println!("Worker en reposo esperando nuevas órdenes...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                } else { //Si se perdio conexion
                    println!("Se perdió la conexión con el coordinador: {:?}", status);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    //Logica de reconexion por implementar
                }
            }
        }
    }
}
