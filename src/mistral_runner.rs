use anyhow::Result;
use futures::StreamExt;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};
use indicatif::{ProgressBar, ProgressStyle};
use mistralrs::{GgufModelBuilder, TextMessages, TextMessageRole, Response};
use reqwest::header::CONTENT_LENGTH;

use async_stream::stream;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;


// download model if missing
pub async fn download_model(repo: &str, file: &str, path: &str) -> Result<()> {
    if Path::new(path).exists() {
        return Ok(());
    }

    println!("Downloading model {file}…");

    let url = format!("https://huggingface.co/{repo}/resolve/main/{file}");
    let response = reqwest::get(&url).await?;

    let total_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )?
    );

    let mut file_out = fs::File::create(path).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file_out.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Download complete.");
    Ok(())
}


// non-streaming inference
pub async fn run_inference_collect(model_name: &str, prompt: &str) -> Result<String> {
    let model_dir = "models";

    //models available: - GGUF
    let models = [
        ("qwen", ("bartowski/Qwen2.5-3B-Instruct-GGUF", "Qwen2.5-3B-Instruct-Q4_K_M.gguf")),
        ("smollm2", ("bartowski/SmolLM2-1.7B-Instruct-GGUF", "smollm2-1.7b-instruct-q4_k_m.gguf")),
        ("llama8b", ("bartowski/Meta-Llama-3.1-8B-Instruct-GGUF", "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf")),
    ];

    let (repo, file) = models
        .iter()
        .find(|m| m.0 == model_name)
        .expect("Unknown model")
        .1;

    let path = format!("{}/{}", model_dir, file);

    download_model(repo, file, path.as_str()).await?;

    let builder = GgufModelBuilder::new(model_dir, vec![file]).with_logging();
    let model = builder.build().await?;

    let messages = TextMessages::new()
        .add_message(TextMessageRole::User, prompt);

    let mut stream = model.stream_chat_request(messages).await?;

    let mut output = String::new();

    while let Some(resp) = stream.next().await {
        if let Response::Chunk(chunk) = resp {
            if let Some(choice) = chunk.choices.get(0) {
                if let Some(text) = &choice.delta.content {
                    output.push_str(text);
                }
            }
        }
    }

    Ok(output)
}



// streaming inference
pub async fn run_inference_stream(
    model_name: &str,
    prompt: &str,
) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>> {

    //download model
    let models = [
        ("qwen", ("bartowski/Qwen2.5-3B-Instruct-GGUF", "Qwen2.5-3B-Instruct-Q4_K_M.gguf")),
        ("smollm2", ("bartowski/SmolLM2-1.7B-Instruct-GGUF", "smollm2-1.7b-instruct-q4_k_m.gguf")),
        ("llama8b", ("bartowski/Meta-Llama-3.1-8B-Instruct-GGUF", "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf")),
    ];

    let (repo, file) = models
        .iter()
        .find(|m| m.0 == model_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown model"))?
        .1;

    let model_dir = "models";
    let path = format!("{}/{}", model_dir, file);

    download_model(repo, file, path.as_str()).await?;

    let builder = GgufModelBuilder::new(model_dir, vec![file]).with_logging();
    let model = Arc::new(builder.build().await?);

    let messages = TextMessages::new()
        .add_message(TextMessageRole::User, prompt);

    let model_for_stream = model.clone();

    let output_stream = stream! {
        let mut mistral_stream = model_for_stream
            .stream_chat_request(messages)
            .await
            .unwrap();

        while let Some(resp) = mistral_stream.next().await {
            if let Response::Chunk(chunk) = resp {
                if let Some(choice) = chunk.choices.get(0) {
                    if let Some(text) = &choice.delta.content {
                        yield text.clone();
                    }
                }
            }
        }
    };

    Ok(Box::pin(output_stream))
}

