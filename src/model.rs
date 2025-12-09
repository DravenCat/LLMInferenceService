//! 模型管理器模块
//!
//! 支持多个 Llama 模型的动态切换:
//! - Llama-3.1-8B-Instruct
//! - Llama-3.2-1B-Instruct  
//! - Llama-3.2-3B-Instruct

#![allow(unused_imports)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};

// ========== 模型枚举 ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelName {
    #[serde(alias = "llama-3.1-8b-instruct", alias = "llama3.1-8b", alias = "llama31-8b")]
    Llama31_8B,
    #[serde(alias = "llama-3.2-1b-instruct", alias = "llama3.2-1b", alias = "llama32-1b")]
    Llama32_1B,
    #[serde(alias = "llama-3.2-3b-instruct", alias = "llama3.2-3b", alias = "llama32-3b")]
    Llama32_3B,
}

impl Default for ModelName {
    fn default() -> Self {
        Self::Llama32_1B // 默认使用最小的模型
    }
}

impl std::fmt::Display for ModelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Llama31_8B => write!(f, "Llama-3.1-8B-Instruct"),
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
            "llama-3.1-8b-instruct" | "llama3.1-8b" | "llama31-8b" | "llama31_8b" => Some(Self::Llama31_8B),
            "llama-3.2-1b-instruct" | "llama3.2-1b" | "llama32-1b" | "llama32_1b" => Some(Self::Llama32_1B),
            "llama-3.2-3b-instruct" | "llama3.2-3b" | "llama32-3b" | "llama32_3b" => Some(Self::Llama32_3B),
            _ => None,
        }
    }

    /// 获取所有可用模型列表
    pub fn available_models() -> Vec<&'static str> {
        vec![
            "llama-3.1-8b-instruct",
            "llama-3.2-1b-instruct",
            "llama-3.2-3b-instruct",
        ]
    }

    /// 获取模型的最大序列长度
    pub fn max_seq_len(&self) -> usize {
        match self {
            Self::Llama31_8B => 8192,   // 8K, 可支持到 128K
            Self::Llama32_1B => 4096,   // 4K, 可支持到 128K
            Self::Llama32_3B => 4096,   // 4K, 可支持到 128K
        }
    }

    /// 获取模型描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Llama31_8B => "Llama 3.1 8B Instruct - 高质量，需要较多显存 (~16GB)",
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationResult {
    pub text: String,
    pub tokens_generated: usize,
    pub generation_time_secs: f64,
    pub model_used: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub loaded: bool,
    pub description: String,
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

// ============================================================================
// 演示实现
// ============================================================================
// 用于测试 HTTP 接口，模拟多模型切换
// 使用真实模型时，注释掉此部分 (约第180-290行)

// pub struct ModelManager {
//     current_model: ModelName,
//     config: GenerationConfig,
// }
//
// impl ModelManager {
//     pub async fn new() -> AppResult<Self> {
//         info!("Initializing DEMO ModelManager with multi-model support...");
//         info!("⚠️  This is a demo implementation.");
//         info!("Available models: {:?}", ModelName::available_models());
//
//         let default_model = ModelName::default();
//         info!("Default model: {}", default_model);
//
//         Ok(Self {
//             current_model: default_model,
//             config: GenerationConfig::default(),
//         })
//     }
//
//     /// 获取当前模型名称
//     pub fn current_model(&self) -> ModelName {
//         self.current_model
//     }
//
//     /// 切换模型
//     pub async fn switch_model(&mut self, model_name: ModelName) -> AppResult<()> {
//         if self.current_model == model_name {
//             info!("Model {} is already loaded", model_name);
//             return Ok(());
//         }
//
//         info!("Switching model from {} to {}...", self.current_model, model_name);
//
//         // 演示模式：直接切换
//         self.current_model = model_name;
//
//         info!("✅ Model switched to {}", model_name);
//         Ok(())
//     }
//
//     /// 获取所有模型信息
//     pub fn list_models(&self) -> Vec<ModelInfo> {
//         vec![
//             ModelInfo {
//                 name: "llama-3.1-8b-instruct".to_string(),
//                 loaded: self.current_model == ModelName::Llama31_8B,
//                 description: ModelName::Llama31_8B.description().to_string(),
//             },
//             ModelInfo {
//                 name: "llama-3.2-1b-instruct".to_string(),
//                 loaded: self.current_model == ModelName::Llama32_1B,
//                 description: ModelName::Llama32_1B.description().to_string(),
//             },
//             ModelInfo {
//                 name: "llama-3.2-3b-instruct".to_string(),
//                 loaded: self.current_model == ModelName::Llama32_3B,
//                 description: ModelName::Llama32_3B.description().to_string(),
//             },
//         ]
//     }
//
//     pub async fn generate(&self, prompt: &str, config: Option<GenerationConfig>) -> AppResult<GenerationResult> {
//         let _cfg = config.unwrap_or_else(|| self.config.clone());
//
//         let response = format!(
//             "[Demo - {}] Response to: '{}'",
//             self.current_model,
//             prompt.chars().take(50).collect::<String>()
//         );
//
//         tokio::time::sleep(Duration::from_millis(100)).await;
//
//         Ok(GenerationResult {
//             text: response,
//             tokens_generated: 20,
//             generation_time_secs: 0.1,
//             model_used: self.current_model.to_string(),
//         })
//     }
//
//     pub async fn stream(&self, prompt: &str, config: Option<GenerationConfig>) -> mpsc::Receiver<StreamChunk> {
//         let (tx, rx) = mpsc::channel(32);
//         let cfg = config.unwrap_or_else(|| self.config.clone());
//         let model_name = self.current_model.to_string();
//
//         let response = format!(
//             "[Demo - {}] Streaming response for: '{}'",
//             model_name,
//             prompt.chars().take(30).collect::<String>()
//         );
//
//         tokio::spawn(async move {
//             let mut generated = String::new();
//             let words: Vec<&str> = response.split_whitespace().collect();
//             let total = words.len().min(cfg.max_new_tokens);
//
//             for (i, word) in words.iter().take(total).enumerate() {
//                 let token_text = if i > 0 { format!(" {}", word) } else { word.to_string() };
//                 generated.push_str(&token_text);
//
//                 let is_last = i == total - 1;
//                 let chunk = StreamChunk {
//                     token_text,
//                     generated_text: generated.clone(),
//                     is_finished: is_last,
//                     finish_reason: if is_last { Some("stop".into()) } else { None },
//                 };
//
//                 if tx.send(chunk).await.is_err() {
//                     break;
//                 }
//
//                 tokio::time::sleep(Duration::from_millis(50)).await;
//             }
//         });
//
//         rx
//     }
//
//     pub fn format_chat_prompt(&self, messages: &[ChatMessage]) -> String {
//         format_llama3_chat(messages)
//     }
// }

// ============================================================================
// llama-burn 多模型真实实现
// ============================================================================
// 要使用真实模型:
// 1. 使用 Cargo.llama-burn.toml 替换 Cargo.toml
// 2. 注释掉上面的 "演示实现" 部分 (约第180-290行)
// 3. 取消下面代码的注释
// ============================================================================


use burn::backend::wgpu::{Vulkan, WgpuDevice};
use burn::tensor::f16;
use llama_burn::{
    llama::LlamaConfig,
    sampling::{Sampler, TopP},
    tokenizer::Tiktoken,
};
use std::sync::Mutex as StdMutex;

// 后端类型
type B = Vulkan<f16, i32>;
type LlamaModel = llama_burn::llama::Llama<B, Tiktoken>;

pub struct ModelManager {
    current_model: Option<Arc<StdMutex<LlamaModel>>>,
    current_model_name: ModelName,
    device: WgpuDevice,
    config: GenerationConfig,
}

impl ModelManager {
    pub async fn new() -> AppResult<Self> {
        info!("Initializing ModelManager with multi-model support...");
        
        let device = WgpuDevice::default();
        info!("Using device: {:?}", device);
        info!("Available models: {:?}", ModelName::available_models());
        
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

        info!("✅ {} loaded successfully!", default_model);

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
        info!("⚠️  This will unload the current model and load the new one.");
        info!("First load may take several minutes to download weights.");
        
        // 释放当前模型
        self.current_model = None;
        
        let device = self.device.clone();
        let max_seq_len = model_name.max_seq_len();
        
        let llama = tokio::task::spawn_blocking(move || {
            match model_name {
                ModelName::Llama31_8B => {
                    LlamaConfig::llama3_1_8b_pretrained::<B>(max_seq_len, &device)
                }
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
        
        info!("✅ Model switched to {}", model_name);
        Ok(())
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                name: "llama-3.1-8b-instruct".to_string(),
                loaded: self.current_model_name == ModelName::Llama31_8B,
                description: ModelName::Llama31_8B.description().to_string(),
            },
            ModelInfo {
                name: "llama-3.2-1b-instruct".to_string(),
                loaded: self.current_model_name == ModelName::Llama32_1B,
                description: ModelName::Llama32_1B.description().to_string(),
            },
            ModelInfo {
                name: "llama-3.2-3b-instruct".to_string(),
                loaded: self.current_model_name == ModelName::Llama32_3B,
                description: ModelName::Llama32_3B.description().to_string(),
            },
        ]
    }

    pub async fn generate(&self, prompt: &str, config: Option<GenerationConfig>) -> AppResult<GenerationResult> {
        let llama = self.current_model.as_ref()
            .ok_or_else(|| AppError::ModelNotLoaded("No model loaded".to_string()))?
            .clone();
        
        let cfg = config.unwrap_or_else(|| self.config.clone());
        let prompt_owned = prompt.to_string();
        let model_name = self.current_model_name.to_string();
        
        let result = tokio::task::spawn_blocking(move || {
            let mut llama = llama.lock().unwrap();
            llama.reset();
            
            let mut sampler = if cfg.temperature > 0.0 {
                Sampler::TopP(TopP::new(cfg.top_p, cfg.seed))
            } else {
                Sampler::Argmax
            };
            
            llama.generate(
                &prompt_owned,
                cfg.max_new_tokens,
                cfg.temperature,
                &mut sampler,
            )
        })
        .await
        .map_err(|e| AppError::GenerationFailed(format!("Task error: {}", e)))?;

        Ok(GenerationResult {
            text: result.text,
            tokens_generated: result.tokens,
            generation_time_secs: result.time,
            model_used: model_name,
        })
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
            llama.reset();
            
            let mut sampler = if cfg.temperature > 0.0 {
                Sampler::TopP(TopP::new(cfg.top_p, cfg.seed))
            } else {
                Sampler::Argmax
            };
            
            let result = llama.generate(
                &prompt_owned,
                cfg.max_new_tokens,
                cfg.temperature,
                &mut sampler,
            );
            
            let mut generated_text = String::new();
            let words: Vec<&str> = result.text.split_whitespace().collect();
            let total = words.len();
            
            for (i, word) in words.into_iter().enumerate() {
                let token_text = if generated_text.is_empty() {
                    word.to_string()
                } else {
                    format!(" {}", word)
                };
                generated_text.push_str(&token_text);
                
                let is_last = i == total - 1;
                let chunk = StreamChunk {
                    token_text,
                    generated_text: generated_text.clone(),
                    is_finished: is_last,
                    finish_reason: if is_last { Some("stop".into()) } else { None },
                };
                
                if tx.blocking_send(chunk).is_err() {
                    break;
                }
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
        assert_eq!(ModelName::from_str("llama-3.1-8b-instruct"), Some(ModelName::Llama31_8B));
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
