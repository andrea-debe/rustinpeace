use std::time::Duration;
use tonic::Request;

// Importamos el código gRPC generado (la versión Cliente)
use mandelbrot_distribuido::mandelbrot::calculator_client::CalculatorClient;
use mandelbrot_distribuido::mandelbrot::{Empty, ResultPayload};

// --- EL MOTOR MATEMÁTICO ---
// Esta función toma el número de fila y calcula el color de todos sus píxeles.
fn calculate_mandelbrot_row(row: u32, width: u32, height: u32, max_iterations: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(width as usize);

    // Límites matemáticos del fractal en el plano cartesiano
    let x_min = -2.0;
    let x_max = 1.0;
    let y_min = -1.5;
    let y_max = 1.5;

    // Convertimos nuestra "fila" de la imagen en una coordenada Y del plano cartesiano
    let y0 = y_min + (row as f64 / height as f64) * (y_max - y_min);

    for col in 0..width {
        // Convertimos el "píxel" actual en una coordenada X del plano cartesiano
        let x0 = x_min + (col as f64 / width as f64) * (x_max - x_min);

        let mut x = 0.0;
        let mut y = 0.0;
        let mut iteration = 0;

        // La famosa fórmula: Z = Z^2 + C
        while x * x + y * y <= 4.0 && iteration < max_iterations {
            let x_temp = x * x - y * y + x0;
            y = 2.0 * x * y + y0;
            x = x_temp;
            iteration += 1;
        }

        // Convertimos las iteraciones en un valor de "color" (un byte de 0 a 255)
        let color_value = (iteration % 256) as u8;
        pixels.push(color_value);
    }

    // Regresamos el arreglo de colores crudos (Bytes)
    pixels
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // IMPORTANTE: Como estás en tu máquina local sin conectar a la VPN por ahora,
    // usaremos 127.0.0.1. Cuando lo lleves a Docker, lo cambiarás por 10.0.0.1
    let coordinator_url = "http://10.0.0.1:3000";
    let my_worker_id = "worker-wsl2"; // Puedes cambiarle el nombre

    println!("Iniciando worker: {}", my_worker_id);
    println!("Buscando al coordinador en {}...", coordinator_url);

    // TOLERANCIA A FALLOS: Un bucle infinito intentando conectar.
    // Si el coordinador se cae o aún no está levantado, el worker no muere, solo espera.
    let mut client = loop {
        match CalculatorClient::connect(coordinator_url).await {
            Ok(c) => {
                println!("¡Conexión gRPC establecida exitosamente!\n");
                break c;
            }
            Err(_) => {
                println!("Esperando al coordinador... Reintentando en 3s...");
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    };

    // EL BUCLE DE TRABAJO
    loop {
        // 1. Pedimos trabajo al coordinador
        let request = Request::new(Empty {});
        
        match client.get_task(request).await {
            Ok(response) => {
                let task = response.into_inner();
                // println!(">> Recibida Tarea: Calcular fila {}", task.row);

                // 2. Ejecutamos el motor matemático (El trabajo pesado real)
                let pixels_data = calculate_mandelbrot_row(
                    task.row,
                    task.width,
                    task.height,
                    task.max_iterations,
                );

                // 3. Empaquetamos los bytes binarios
                let result_payload = ResultPayload {
                    task_id: task.task_id,
                    worker_id: my_worker_id.to_string(),
                    data: pixels_data,
                };

                // 4. Enviamos el resultado por el túnel gRPC
                let result_req = Request::new(result_payload);
                if let Err(e) = client.submit_result(result_req).await {
                    println!("Error enviando resultado al coordinador: {:?}", e);
                } else {
                    println!("<< Fila {} calculada y enviada", task.row);
                }
            }
            Err(status) => {
                // Si el coordinador responde "NOT_FOUND", significa que ya se calculó toda la imagen
                if status.code() == tonic::Code::NotFound {
                    println!("\n¡El coordinador informa que el fractal está completo!");
                    println!("Worker en reposo esperando nuevas órdenes...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                } else {
                    println!("Se perdió la conexión con el coordinador: {:?}", status);
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    // Aquí podrías agregar lógica para intentar reconectar
                }
            }
        }
    }
}
