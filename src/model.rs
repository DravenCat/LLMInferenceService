use burn;
use std::sync::Arc;
use burn_wgpu::{WgpuDevice};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use crate::tokenizer::Tokenizer;


struct ModelConfig {

}


impl ModelConfig {
    fn new() -> Self {


        Self {

        }
    }
}


struct Model {

}


impl Model {
    fn new(model_path : &str, _device : &WgpuDevice) -> Self {


        Self {

        }
    }
}


pub struct ModelManager {
    model: Arc<Model>,
    tokenizer: Arc<Tokenizer>,
    device: WgpuDevice,
}


// Not streaming
#[derive(Serialize, Deserialize)]
pub struct GenerationResult {
    pub text: String,
    pub tokens : Vec<u32>,
    pub length : usize,
    pub inference_time : f64,
}


// Streaming
#[derive(Serialize, Deserialize)]
pub struct StreamResult {
    pub token : u32,
    pub text: String,
    pub generated_text: String,
    pub progress: f32,
    pub is_end : bool,
}


impl ModelManager {
    pub async fn new(model_path: &str, tokenizer_path : &str) -> Self {
        let device = WgpuDevice::default();

        let tokenizer = Tokenizer::new(tokenizer_path);

        let model = Model::new(model_path, &device);


        Self {
            model : Arc::new(model),
            tokenizer : Arc::new(tokenizer),
            device,
        }
    }


    pub async fn generate(&self, prompt : &str) -> GenerationResult {
        let start_time = std::time::Instant::now();



        let inference_time = start_time.elapsed().as_secs_f64();

        GenerationResult {
            text : String::from(prompt),
            tokens : vec![1, 2, 3, 4, 5],
            length : 0,
            inference_time,
        }
    }


    pub async fn stream(&self, prompt : &str) -> mpsc::Receiver<StreamResult> {
        let (tx, rx) = mpsc::channel(32);

        rx
    }
}