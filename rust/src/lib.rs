use serde::{Deserialize, Serialize};

// Esto es lo que el Coordinador envía al Worker
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub task_id: u32,
    pub instruction: String, 
}

// Esto es lo que el Worker le responde al Coordinador
#[derive(Serialize, Deserialize, Debug)]
pub struct ResultPayload {
    pub task_id: u32,
    pub worker_id: String,
    pub data: String, 
}