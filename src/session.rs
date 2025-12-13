use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// 单条消息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// 会话配置
#[derive(Clone)]
pub struct SessionConfig {

    pub max_turns: usize,

    pub system_prompt: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_turns: 10,
            system_prompt: None,
        }
    }
}


#[derive(Clone)]
pub struct Session {
    pub id: String,
    pub messages: Vec<ChatMessage>,
    pub config: SessionConfig,
}

impl Session {
    pub fn new(id: String, config: SessionConfig) -> Self {
        let mut messages = Vec::new();


        if let Some(system_prompt) = &config.system_prompt {
            messages.push(ChatMessage {
                role: MessageRole::System,
                content: system_prompt.clone(),
            });
        }

        Self { id,
            messages,
            config
        }
    }


    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content,
        });
        self.trim_history();
    }


    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content,
        });
        self.trim_history();
    }


    pub fn get_messages(&self) -> &[ChatMessage] {
        &self.messages
    }


    pub fn clear(&mut self) {
        let system_msg = self.messages.iter()
            .find(|m| m.role == MessageRole::System)
            .cloned();

        self.messages.clear();

        if let Some(msg) = system_msg {
            self.messages.push(msg);
        }
    }


    fn trim_history(&mut self) {

        let non_system_messages: Vec<_> = self.messages.iter()
            .filter(|m| m.role != MessageRole::System)
            .collect();


        let current_turns = non_system_messages.len() / 2;

        if current_turns > self.config.max_turns {
            let messages_to_remove = (current_turns - self.config.max_turns) * 2;


            let first_non_system_idx = self.messages.iter()
                .position(|m| m.role != MessageRole::System)
                .unwrap_or(0);


            self.messages.drain(first_non_system_idx..first_non_system_idx + messages_to_remove);
        }
    }
}


pub type SessionManager = Arc<RwLock<HashMap<String, Session>>>;

pub fn new_session_manager() -> SessionManager {
    Arc::new(RwLock::new(HashMap::new()))
}


pub struct SessionHelper;

impl SessionHelper {

    pub async fn get_or_create(
        manager: &SessionManager,
        session_id: &str,
        config: SessionConfig,
    ) -> Session {
        let mut sessions = manager.write().await;

        sessions.entry(session_id.to_string())
            .or_insert_with(|| Session::new(session_id.to_string(), config))
            .clone()
    }


    pub async fn update(manager: &SessionManager, session: Session) {
        let mut sessions = manager.write().await;
        sessions.insert(session.id.clone(), session);
    }


    pub async fn remove(manager: &SessionManager, session_id: &str) -> bool {
        let mut sessions = manager.write().await;
        match sessions.get(session_id) {
            Some(_) => {
                sessions.remove(session_id);
            },
            None => {
                return false
            }
        }

        true
    }


    pub async fn clear_history(manager: &SessionManager, session_id: &str) {
        let mut sessions = manager.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.clear();
        }
    }
}