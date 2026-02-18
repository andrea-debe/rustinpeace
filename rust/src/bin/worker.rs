use mandelbrot_distribuido::{ResultPayload, Task};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let coordinator_url = "http://127.0.0.1:3000"; 
    let my_worker_id = "worker-01";

    println!("Iniciando worker: {}...", my_worker_id);

    loop {
        // 1. Pedir tarea al Coordinador
        let res = client.get(&format!("{}/task", coordinator_url)).send().await;
        
        match res {
            Ok(response) if response.status().is_success() => {
                let task: Task = response.json().await?;
                println!(">> Tarea recibida: {:?}", task);

                // 2. Simulamos que estamos haciendo cálculos pesados
                println!("Procesando tarea...");
                tokio::time::sleep(Duration::from_secs(2)).await;
                let result_data = format!("Cálculo completado para tarea {}", task.task_id);

                // 3. Empaquetar el resultado y enviarlo
                let payload = ResultPayload {
                    task_id: task.task_id,
                    worker_id: my_worker_id.to_string(),
                    data: result_data,
                };

                let _ = client.post(&format!("{}/result", coordinator_url))
                    .json(&payload)
                    .send()
                    .await?;
                
                println!("<< Resultado enviado al coordinador.\n");
            },
            _ => {
                println!("No se pudo contactar al coordinador. Reintentando...");
            }
        }
        
        // Pausa de 2 segundos antes de pedir otra tarea
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}