use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};


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

    /// 获取 session（如果存在）
    pub async fn get(manager: &SessionManager, session_id: &str) -> Option<Session> {
        let sessions = manager.read().await;
        sessions.get(session_id).cloned()
    }

    /// 同步 session 消息（从前端恢复历史）
    pub async fn sync_messages(
        manager: &SessionManager,
        session_id: &str,
        messages: Vec<ChatMessage>,
        config: SessionConfig,
    ) -> Session {
        let mut sessions = manager.write().await;
        
        // 创建或更新 session
        let session = sessions.entry(session_id.to_string())
            .or_insert_with(|| Session::new(session_id.to_string(), config.clone()));
        
        // 替换消息历史
        session.messages = messages;
        
        // 应用消息数量限制
        session.config = config;
        session.trim_history();
        
        session.clone()
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
                {
                    println!("Number of alive session {}", sessions.len());
                }
            },
            None => {
                return false
            }
        }

        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.max_turns, 10);
        assert!(config.system_prompt.is_none());
    }

    #[test]
    fn test_session_config_custom() {
        let config = SessionConfig {
            max_turns: 5,
            system_prompt: Some("You are a helpful assistant.".to_string()),
        };
        assert_eq!(config.max_turns, 5);
        assert_eq!(config.system_prompt, Some("You are a helpful assistant.".to_string()));
    }

    #[test]
    fn test_session_new_without_system_prompt() {
        let config = SessionConfig {
            max_turns: 10,
            system_prompt: None,
        };
        let session = Session::new("test-id".to_string(), config);

        assert_eq!(session.id, "test-id");
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_new_with_system_prompt() {
        let config = SessionConfig {
            max_turns: 10,
            system_prompt: Some("System prompt".to_string()),
        };
        let session = Session::new("test-id".to_string(), config);

        assert_eq!(session.id, "test-id");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, MessageRole::System);
        assert_eq!(session.messages[0].content, "System prompt");
    }

    #[test]
    fn test_add_user_message() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Hello".to_string());

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, MessageRole::User);
        assert_eq!(session.messages[0].content, "Hello");
    }

    #[test]
    fn test_add_assistant_message() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        session.add_assistant_message("Hi there!".to_string());

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, MessageRole::Assistant);
        assert_eq!(session.messages[0].content, "Hi there!");
    }

    #[test]
    fn test_add_multiple_messages() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Question 1".to_string());
        session.add_assistant_message("Answer 1".to_string());
        session.add_user_message("Question 2".to_string());
        session.add_assistant_message("Answer 2".to_string());

        assert_eq!(session.messages.len(), 4);
        assert_eq!(session.messages[0].role, MessageRole::User);
        assert_eq!(session.messages[1].role, MessageRole::Assistant);
        assert_eq!(session.messages[2].role, MessageRole::User);
        assert_eq!(session.messages[3].role, MessageRole::Assistant);
    }

    #[test]
    fn test_add_messages_with_system_prompt() {
        let config = SessionConfig {
            max_turns: 10,
            system_prompt: Some("System".to_string()),
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Hello".to_string());
        session.add_assistant_message("Hi".to_string());

        assert_eq!(session.messages.len(), 3);
        assert_eq!(session.messages[0].role, MessageRole::System);
        assert_eq!(session.messages[1].role, MessageRole::User);
        assert_eq!(session.messages[2].role, MessageRole::Assistant);
    }

    #[test]
    fn test_trim_history_under_limit() {
        let config = SessionConfig {
            max_turns: 3,
            system_prompt: None,
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.add_user_message("Q2".to_string());
        session.add_assistant_message("A2".to_string());

        assert_eq!(session.messages.len(), 4);
    }

    #[test]
    fn test_trim_history_at_limit() {
        let config = SessionConfig {
            max_turns: 2,
            system_prompt: None,
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.add_user_message("Q2".to_string());
        session.add_assistant_message("A2".to_string());

        assert_eq!(session.messages.len(), 4);
    }

    #[test]
    fn test_trim_history_over_limit() {
        let config = SessionConfig {
            max_turns: 2,
            system_prompt: None,
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.add_user_message("Q2".to_string());
        session.add_assistant_message("A2".to_string());
        session.add_user_message("Q3".to_string());
        session.add_assistant_message("A3".to_string());

        assert_eq!(session.messages.len(), 4);
        assert_eq!(session.messages[0].content, "Q2");
        assert_eq!(session.messages[1].content, "A2");
        assert_eq!(session.messages[2].content, "Q3");
        assert_eq!(session.messages[3].content, "A3");
    }

    #[test]
    fn test_trim_history_preserves_system_prompt() {
        let config = SessionConfig {
            max_turns: 2,
            system_prompt: Some("System".to_string()),
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.add_user_message("Q2".to_string());
        session.add_assistant_message("A2".to_string());
        session.add_user_message("Q3".to_string());
        session.add_assistant_message("A3".to_string());

        assert_eq!(session.messages.len(), 5);
        assert_eq!(session.messages[0].role, MessageRole::System);
        assert_eq!(session.messages[0].content, "System");
        assert_eq!(session.messages[1].content, "Q2");
        assert_eq!(session.messages[4].content, "A3");
    }

    #[test]
    fn test_trim_history_single_turn() {
        let config = SessionConfig {
            max_turns: 1,
            system_prompt: None,
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.add_user_message("Q2".to_string());
        session.add_assistant_message("A2".to_string());
        session.add_user_message("Q3".to_string());
        session.add_assistant_message("A3".to_string());

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].content, "Q3");
        assert_eq!(session.messages[1].content, "A3");
    }


    #[test]
    fn test_clear_without_system_prompt() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.clear();

        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_clear_with_system_prompt() {
        let config = SessionConfig {
            max_turns: 10,
            system_prompt: Some("System prompt".to_string()),
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());
        session.clear();

        // 只保留系统消息
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, MessageRole::System);
        assert_eq!(session.messages[0].content, "System prompt");
    }


    #[test]
    fn test_get_messages() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Hello".to_string());
        session.add_assistant_message("Hi".to_string());

        let messages = session.get_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi");
    }


    #[test]
    fn test_new_session_manager() {
        let manager = new_session_manager();
        let guard = manager.try_write();
        assert!(guard.is_ok());
    }

    #[tokio::test]
    async fn test_helper_get_or_create_new_session() {
        let manager = new_session_manager();
        let config = SessionConfig::default();

        let session = SessionHelper::get_or_create(&manager, "session-1", config).await;

        assert_eq!(session.id, "session-1");
        assert!(session.messages.is_empty());
    }

    #[tokio::test]
    async fn test_helper_get_or_create_existing_session() {
        let manager = new_session_manager();
        let config = SessionConfig::default();

        let mut session = SessionHelper::get_or_create(&manager, "session-1", config.clone()).await;
        session.add_user_message("Hello".to_string());
        SessionHelper::update(&manager, session).await;

        let session = SessionHelper::get_or_create(&manager, "session-1", config).await;

        assert_eq!(session.id, "session-1");
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_helper_update() {
        let manager = new_session_manager();
        let config = SessionConfig::default();

        let mut session = Session::new("session-1".to_string(), config);
        session.add_user_message("Test".to_string());

        SessionHelper::update(&manager, session).await;

        let sessions = manager.read().await;
        assert!(sessions.contains_key("session-1"));
        assert_eq!(sessions.get("session-1").unwrap().messages.len(), 1);
    }

    #[tokio::test]
    async fn test_helper_remove() {
        let manager = new_session_manager();
        let config = SessionConfig::default();

        let session = SessionHelper::get_or_create(&manager, "session-1", config).await;
        SessionHelper::update(&manager, session).await;

        SessionHelper::remove(&manager, "session-1").await;

        let sessions = manager.read().await;
        assert!(!sessions.contains_key("session-1"));
    }

    #[tokio::test]
    async fn test_helper_remove_nonexistent() {
        let manager = new_session_manager();

        SessionHelper::remove(&manager, "nonexistent").await;

        let sessions = manager.read().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let manager = new_session_manager();
        let config = SessionConfig::default();

        let mut session1 = SessionHelper::get_or_create(&manager, "session-1", config.clone()).await;
        let mut session2 = SessionHelper::get_or_create(&manager, "session-2", config.clone()).await;

        session1.add_user_message("Hello from 1".to_string());
        session2.add_user_message("Hello from 2".to_string());

        SessionHelper::update(&manager, session1).await;
        SessionHelper::update(&manager, session2).await;

        let sessions = manager.read().await;
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions.get("session-1").unwrap().messages[0].content, "Hello from 1");
        assert_eq!(sessions.get("session-2").unwrap().messages[0].content, "Hello from 2");
    }


    #[test]
    fn test_message_role_equality() {
        assert_eq!(MessageRole::User, MessageRole::User);
        assert_eq!(MessageRole::Assistant, MessageRole::Assistant);
        assert_eq!(MessageRole::System, MessageRole::System);
        assert_ne!(MessageRole::User, MessageRole::Assistant);
    }


    #[test]
    fn test_empty_message_content() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("".to_string());

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "");
    }

    #[test]
    fn test_long_message_content() {
        let config = SessionConfig::default();
        let mut session = Session::new("test".to_string(), config);

        let long_content = "a".repeat(10000);
        session.add_user_message(long_content.clone());

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, long_content);
    }


    #[test]
    fn test_max_turns_zero() {
        let config = SessionConfig {
            max_turns: 0,
            system_prompt: None,
        };
        let mut session = Session::new("test".to_string(), config);

        session.add_user_message("Q1".to_string());
        session.add_assistant_message("A1".to_string());

        assert!(session.messages.is_empty());
    }
}
