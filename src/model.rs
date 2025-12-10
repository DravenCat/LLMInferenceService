//! 模型管理器模块
//!
//! 支持多个 Llama 模型的动态切换:
//! - Llama-3.2-1B-Instruct
//! - Llama-3.2-3B-Instruct

#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

use burn::backend::wgpu::{Vulkan, WgpuDevice};
use burn::tensor::f16;
use burn::tensor::{backend::Backend, Device, Int, Shape, Tensor, TensorData, ElementConversion};
use burn::tensor::activation::softmax;
use burn::tensor::cast::ToElement;
use llama_burn::{
    llama::LlamaConfig,
    sampling::{Sampler, TopP},
    tokenizer::Tiktoken,
};
use llama_burn::tokenizer::Tokenizer;
use std::sync::Mutex as StdMutex;

use crate::error::{AppError, AppResult};

// ========== 模型枚举 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelName {
    #[serde(alias = "llama-3.2-1b-instruct", alias = "llama3.2-1b", alias = "llama32-1b")]
    Llama32_1B,
    #[serde(alias = "llama-3.2-3b-instruct", alias = "llama3.2-3b", alias = "llama32-3b")]
    Llama32_3B,
}

impl Default for ModelName {
    fn default() -> Self {
        Self::Llama32_1B
    }
}

impl std::fmt::Display for ModelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Llama32_1B => write!(f, "Llama-3.2-1B-Instruct"),
            Self::Llama32_3B => write!(f, "Llama-3.2-3B-Instruct"),
        }
    }
}

impl ModelName {
    /// 从字符串解析模型名称
    pub fn from_str(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "llama-3.2-1b-instruct" | "llama3.2-1b" | "llama32-1b" | "llama32_1b" => Some(Self::Llama32_1B),
            "llama-3.2-3b-instruct" | "llama3.2-3b" | "llama32-3b" | "llama32_3b" => Some(Self::Llama32_3B),
            _ => None,
        }
    }

    /// 获取所有可用模型列表
    pub fn available_models() -> Vec<&'static str> {
        vec![
            "llama-3.2-1b-instruct",
            "llama-3.2-3b-instruct",
        ]
    }

    /// 获取模型的最大序列长度
    pub fn max_seq_len(&self) -> usize {
        match self {
            Self::Llama32_1B => 4096,   // 4K, 可支持到 128K
            Self::Llama32_3B => 4096,   // 4K, 可支持到 128K
        }
    }

    /// 获取模型描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Llama32_1B => "Llama 3.2 1B Instruct - 轻量级，适合资源受限环境 (~4GB)",
            Self::Llama32_3B => "Llama 3.2 3B Instruct - 平衡性能与资源 (~8GB)",
        }
    }
}

// ========== 配置结构 ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub max_new_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub seed: u64,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 256,
            temperature: 0.6,
            top_p: 0.9,
            seed: 42,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub token_text: String,
    pub generated_text: String,
    pub is_finished: bool,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

// ========== Llama-3 Chat 模板 ==========

pub fn format_llama3_chat(messages: &[ChatMessage]) -> String {
    let mut prompt = String::new();
    
    prompt.push_str("<|begin_of_text|>");
    
    let has_system = messages.iter().any(|m| m.role == "system");
    if !has_system {
        prompt.push_str("<|start_header_id|>system<|end_header_id|>\n\n");
        prompt.push_str("You are a helpful assistant.");
        prompt.push_str("<|eot_id|>");
    }
    
    for msg in messages {
        match msg.role.as_str() {
            "system" => {
                prompt.push_str("<|start_header_id|>system<|end_header_id|>\n\n");
                prompt.push_str(&msg.content);
                prompt.push_str("<|eot_id|>");
            }
            "user" => {
                prompt.push_str("<|start_header_id|>user<|end_header_id|>\n\n");
                prompt.push_str(&msg.content);
                prompt.push_str("<|eot_id|>");
            }
            "assistant" => {
                prompt.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
                prompt.push_str(&msg.content);
                prompt.push_str("<|eot_id|>");
            }
            _ => {
                prompt.push_str("<|start_header_id|>user<|end_header_id|>\n\n");
                prompt.push_str(&msg.content);
                prompt.push_str("<|eot_id|>");
            }
        }
    }
    
    prompt.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
    prompt
}


// 后端类型
type B = Vulkan<f16, i32>;
type LlamaModel = llama_burn::llama::Llama<B, Tiktoken>;

pub struct ModelManager {
    current_model: Option<Arc<StdMutex<LlamaModel>>>,
    current_model_name: ModelName,
    device: WgpuDevice,
    config: GenerationConfig,
}

/// Temperature-scaled softmax
fn temperature_scaled_softmax<Backend: burn::tensor::backend::Backend>(
    logits: Tensor<Backend, 2>,
    temperature: f64,
) -> Tensor<Backend, 2> {
    softmax(logits / temperature, 1)
}

impl ModelManager {
    pub async fn new() -> AppResult<Self> {

        let device = WgpuDevice::default();
        info!("Using device: {:?}", device);

        // 默认加载最小的模型
        let default_model = ModelName::Llama32_1B;
        info!("Loading default model: {}...", default_model);
        
        let device_clone = device.clone();
        let max_seq_len = default_model.max_seq_len();
        
        let llama = tokio::task::spawn_blocking(move || {
            LlamaConfig::llama3_2_1b_pretrained::<B>(max_seq_len, &device_clone)
        })
        .await
        .map_err(|e| AppError::ModelNotLoaded(format!("Task panic: {}", e)))?
        .map_err(|e| AppError::ModelNotLoaded(format!("Model error: {}", e)))?;

        info!("{} loaded successfully!", default_model);

        Ok(Self {
            current_model: Some(Arc::new(StdMutex::new(llama))),
            current_model_name: default_model,
            device,
            config: GenerationConfig::default(),
        })
    }

    pub fn current_model(&self) -> ModelName {
        self.current_model_name
    }

    pub async fn switch_model(&mut self, model_name: ModelName) -> AppResult<()> {
        if self.current_model_name == model_name {
            info!("Model {} is already loaded", model_name);
            return Ok(());
        }
        
        info!("Switching model from {} to {}...", self.current_model_name, model_name);
        info!("This will unload the current model and load the new one.");
        info!("First load may take several minutes to download weights.");
        
        // 释放当前模型
        self.current_model = None;
        
        let device = self.device.clone();
        let max_seq_len = model_name.max_seq_len();
        
        let llama = tokio::task::spawn_blocking(move || {
            match model_name {
                ModelName::Llama32_1B => {
                    LlamaConfig::llama3_2_1b_pretrained::<B>(max_seq_len, &device)
                }
                ModelName::Llama32_3B => {
                    LlamaConfig::llama3_2_3b_pretrained::<B>(max_seq_len, &device)
                }
            }
        })
        .await
        .map_err(|e| AppError::ModelNotLoaded(format!("Task panic: {}", e)))?
        .map_err(|e| AppError::ModelNotLoaded(format!("Failed to load {}: {}", model_name, e)))?;

        self.current_model = Some(Arc::new(StdMutex::new(llama)));
        self.current_model_name = model_name;
        
        info!("Model switched to {}", model_name);
        Ok(())
    }


    pub async fn stream(&self, prompt: &str, config: Option<GenerationConfig>) -> mpsc::Receiver<StreamChunk> {
        let (tx, rx) = mpsc::channel(32);
        
        let llama = match self.current_model.as_ref() {
            Some(m) => m.clone(),
            None => {
                // 发送错误并返回
                tokio::spawn(async move {
                    let _ = tx.send(StreamChunk {
                        token_text: "Error: No model loaded".to_string(),
                        generated_text: "Error: No model loaded".to_string(),
                        is_finished: true,
                        finish_reason: Some("error".to_string()),
                    }).await;
                });
                return rx;
            }
        };
        
        let cfg = config.unwrap_or_else(|| self.config.clone());
        let prompt_owned = prompt.to_string();

        tokio::task::spawn_blocking(move || {
            let mut llama = llama.lock().unwrap();

            // 重置 KV cache
            llama.cache.iter_mut().for_each(|cache| cache.reset());

            // 创建采样器
            let mut sampler = if cfg.temperature > 0.0 {
                Sampler::TopP(TopP::new(cfg.top_p, cfg.seed))
            } else {
                Sampler::Argmax
            };

            // Tokenize prompt
            let input_tokens = llama.tokenizer.encode(&prompt_owned, true, false);
            let prompt_len = input_tokens.len();

            // 克隆 device 以避免借用冲突
            let device = llama.device.clone();

            // 获取 stop tokens (在进入循环前完成所有 tokenizer 操作)
            let stop_token_ids = llama.tokenizer.stop_ids();

            // 创建 tokens tensor
            let total_len = prompt_len + cfg.max_new_tokens;
            let mut tokens = Tensor::<B, 1, Int>::empty([total_len], &device);

            // 填入 prompt tokens
            let input_tensor = Tensor::<B, 1, Int>::from_data(
                TensorData::new(input_tokens.clone(), Shape::new([prompt_len])),
                &device
            );
            tokens = tokens.slice_assign([0..prompt_len], input_tensor);

            // 创建 stop tokens tensor
            let stop_tokens = Tensor::from_ints(stop_token_ids.as_slice(), &device);

            let mut generated_text = String::new();
            let mut num_tokens: usize = 0;
            let mut input_pos = Tensor::<B, 1, Int>::arange(0..prompt_len as i64, &device);

            // ====== 真正的流式生成循环 ======
            for i in 0..cfg.max_new_tokens {
                // 前向传播 - 使用解构来分离借用
                let x = tokens.clone().select(0, input_pos.clone()).reshape([1, -1]);

                // 通过解构获取可变和不可变引用
                let llama_ref = &mut *llama;
                let logits = llama_ref.model.forward(x, &mut llama_ref.cache, &llama_ref.rope);

                let [batch_size, seq_len, _vocab_size] = logits.dims();
                let mut next_token_logits = logits
                    .slice([0..batch_size, seq_len - 1..seq_len])
                    .squeeze_dim(1);

                // 应用 temperature
                if cfg.temperature > 0.0 {
                    next_token_logits = temperature_scaled_softmax(next_token_logits, cfg.temperature);
                }

                // 采样下一个 token
                let next_token = sampler.sample(next_token_logits).squeeze_dim(0);

                // 检查是否是 stop token
                let is_stop = stop_tokens
                    .clone()
                    .equal(next_token.clone())
                    .any()
                    .into_scalar()
                    .to_bool();

                if is_stop {
                    // 发送最终 chunk
                    let _ = tx.blocking_send(StreamChunk {
                        token_text: String::new(),
                        generated_text: generated_text.clone(),
                        is_finished: true,
                        finish_reason: Some("stop".to_string()),
                    });
                    break;
                }

                // 获取 token ID 并解码为文本
                let token_id = next_token.clone().into_data().as_slice::<i32>().unwrap()[0] as u32;
                let token_text = llama_ref.tokenizer.decode(vec![token_id]);

                // 更新 tokens tensor
                tokens = tokens.slice_assign([prompt_len + i..prompt_len + i + 1], next_token);
                num_tokens += 1;

                // 累加生成的文本
                generated_text.push_str(&token_text);

                // ====== 立即发送这个 token ======
                let is_last = i == cfg.max_new_tokens - 1;
                let chunk = StreamChunk {
                    token_text,
                    generated_text: generated_text.clone(),
                    is_finished: is_last,
                    finish_reason: if is_last { Some("length".to_string()) } else { None },
                };

                if tx.blocking_send(chunk).is_err() {
                    // 接收端已关闭，停止生成
                    break;
                }

                // 更新位置
                let t = input_pos.dims()[0];
                input_pos = input_pos.slice([t - 1..t]) + 1;
            }

            // 如果循环正常结束且还没发送 is_finished
            if num_tokens == cfg.max_new_tokens {
                let _ = tx.blocking_send(StreamChunk {
                    token_text: String::new(),
                    generated_text,
                    is_finished: true,
                    finish_reason: Some("length".to_string()),
                });
            }
        });

        rx
    }

    pub fn format_chat_prompt(&self, messages: &[ChatMessage]) -> String {
        format_llama3_chat(messages)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_name_from_str() {
        assert_eq!(ModelName::from_str("llama-3.2-1b-instruct"), Some(ModelName::Llama32_1B));
        assert_eq!(ModelName::from_str("llama3.2-1b"), Some(ModelName::Llama32_1B));
        assert_eq!(ModelName::from_str("invalid"), None);
    }

    #[test]
    fn test_llama3_chat_format() {
        let messages = vec![
            ChatMessage { role: "user".into(), content: "Hi!".into() },
        ];
        let prompt = format_llama3_chat(&messages);
        assert!(prompt.starts_with("<|begin_of_text|>"));
        assert!(prompt.contains("You are a helpful assistant.")); // 默认 system
        assert!(prompt.contains("Hi!"));
    }
}