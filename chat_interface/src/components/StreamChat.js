import React, { useState, useRef, useEffect } from "react";

const StreamingChat = () => {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 150) + "px";
    }
  }, [input]);

  const handleSubmit = async () => {
    if (!input.trim() || isStreaming) return;

    const userMessage = { role: "user", content: input.trim() };
    setMessages((prev) => [...prev, userMessage]);
    setInput("");
    setIsStreaming(true);

    // 添加空的assistant消息用于流式更新
    setMessages((prev) => [...prev, { role: "assistant", content: "" }]);

    try {
      const response = await fetch("/api/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          messages: [...messages, userMessage],
        }),
      });

      if (!response.ok) throw new Error("请求失败");

      const reader = response.body.getReader();
      const decoder = new TextDecoder();

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value, { stream: true });

        // 解析SSE格式数据
        const lines = chunk.split("\n");
        for (const line of lines) {
          if (line.startsWith("data: ")) {
            const data = line.slice(6);
            if (data === "[DONE]") continue;

            try {
              const parsed = JSON.parse(data);
              const content = parsed.choices?.[0]?.delta?.content || "";

              if (content) {
                setMessages((prev) => {
                  const newMessages = [...prev];
                  const lastMessage = newMessages[newMessages.length - 1];
                  lastMessage.content += content;
                  return newMessages;
                });
              }
            } catch (e) {
              // 非JSON格式，直接追加文本
              setMessages((prev) => {
                const newMessages = [...prev];
                const lastMessage = newMessages[newMessages.length - 1];
                lastMessage.content += data;
                return newMessages;
              });
            }
          }
        }
      }
    } catch (error) {
      console.error("流式请求错误:", error);
      setMessages((prev) => {
        const newMessages = [...prev];
        const lastMessage = newMessages[newMessages.length - 1];
        lastMessage.content = "抱歉，发生了错误，请重试。";
        return newMessages;
      });
    } finally {
      setIsStreaming(false);
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <div style={styles.headerContent}>
          <div style={styles.logo}>
            <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
              <circle cx="14" cy="14" r="12" stroke="#e8e4df" strokeWidth="2" />
              <circle cx="14" cy="14" r="6" fill="#e8e4df" />
            </svg>
          </div>
          <span style={styles.title}>对话</span>
        </div>
      </div>

      <div style={styles.messagesContainer}>
        {messages.length === 0 ? (
          <div style={styles.emptyState}>
            <div style={styles.emptyIcon}>
              <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
                <path
                  d="M24 4L28.5 15.5L40 20L28.5 24.5L24 36L19.5 24.5L8 20L19.5 15.5L24 4Z"
                  stroke="#4a4540"
                  strokeWidth="2"
                  strokeLinejoin="round"
                />
              </svg>
            </div>
            <p style={styles.emptyText}>开始新对话</p>
            <p style={styles.emptySubtext}>输入消息开始与AI助手交流</p>
          </div>
        ) : (
          <div style={styles.messagesList}>
            {messages.map((msg, idx) => (
              <div
                key={idx}
                style={{
                  ...styles.messageWrapper,
                  ...(msg.role === "user" ? styles.userWrapper : styles.assistantWrapper),
                }}
              >
                <div
                  style={{
                    ...styles.message,
                    ...(msg.role === "user" ? styles.userMessage : styles.assistantMessage),
                  }}
                >
                  {msg.role === "assistant" && (
                    <div style={styles.assistantIcon}>
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <circle cx="8" cy="8" r="6" stroke="#c9a87c" strokeWidth="1.5" />
                        <circle cx="8" cy="8" r="2.5" fill="#c9a87c" />
                      </svg>
                    </div>
                  )}
                  <div style={styles.messageContent}>
                    {msg.content}
                    {msg.role === "assistant" && isStreaming && idx === messages.length - 1 && (
                      <span style={styles.cursor}>▊</span>
                    )}
                  </div>
                </div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      <div style={styles.inputArea}>
        <div style={styles.inputWrapper}>
          <textarea
            ref={textareaRef}
            style={styles.textarea}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入消息..."
            rows={1}
            disabled={isStreaming}
          />
          <button
            style={{
              ...styles.sendButton,
              ...(isStreaming || !input.trim() ? styles.sendButtonDisabled : {}),
            }}
            onClick={handleSubmit}
            disabled={isStreaming || !input.trim()}
          >
            {isStreaming ? (
              <div style={styles.loadingDots}>
                <span style={{ ...styles.dot, animationDelay: "0ms" }}>•</span>
                <span style={{ ...styles.dot, animationDelay: "150ms" }}>•</span>
                <span style={{ ...styles.dot, animationDelay: "300ms" }}>•</span>
              </div>
            ) : (
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                <path
                  d="M3 10H17M17 10L12 5M17 10L12 15"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            )}
          </button>
        </div>
        <p style={styles.hint}>按 Enter 发送，Shift + Enter 换行</p>
      </div>

      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Noto+Serif+SC:wght@400;500;600&family=Noto+Sans+SC:wght@400;500&display=swap');
        
        @keyframes blink {
          0%, 50% { opacity: 1; }
          51%, 100% { opacity: 0; }
        }
        
        @keyframes bounce {
          0%, 100% { transform: translateY(0); }
          50% { transform: translateY(-4px); }
        }
        
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(8px); }
          to { opacity: 1; transform: translateY(0); }
        }
        
        * {
          box-sizing: border-box;
        }
        
        body {
          margin: 0;
          padding: 0;
        }
      `}</style>
    </div>
  );
};

const styles = {
  container: {
    display: "flex",
    flexDirection: "column",
    height: "100vh",
    width: "100%",
    background: "linear-gradient(180deg, #1a1816 0%, #0f0e0d 100%)",
    fontFamily: "'Noto Sans SC', -apple-system, sans-serif",
    color: "#e8e4df",
  },
  header: {
    padding: "16px 24px",
    borderBottom: "1px solid rgba(232, 228, 223, 0.08)",
    background: "rgba(26, 24, 22, 0.8)",
    backdropFilter: "blur(12px)",
  },
  headerContent: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
    maxWidth: "800px",
    margin: "0 auto",
  },
  logo: {
    opacity: 0.9,
  },
  title: {
    fontFamily: "'Noto Serif SC', serif",
    fontSize: "18px",
    fontWeight: 500,
    letterSpacing: "0.05em",
  },
  messagesContainer: {
    flex: 1,
    overflowY: "auto",
    padding: "24px",
    scrollBehavior: "smooth",
  },
  emptyState: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    height: "100%",
    opacity: 0.6,
  },
  emptyIcon: {
    marginBottom: "16px",
    opacity: 0.5,
  },
  emptyText: {
    fontFamily: "'Noto Serif SC', serif",
    fontSize: "20px",
    margin: "0 0 8px 0",
    color: "#e8e4df",
  },
  emptySubtext: {
    fontSize: "14px",
    margin: 0,
    color: "#8a857e",
  },
  messagesList: {
    maxWidth: "800px",
    margin: "0 auto",
    display: "flex",
    flexDirection: "column",
    gap: "20px",
  },
  messageWrapper: {
    display: "flex",
    animation: "fadeIn 0.3s ease-out",
  },
  userWrapper: {
    justifyContent: "flex-end",
  },
  assistantWrapper: {
    justifyContent: "flex-start",
  },
  message: {
    maxWidth: "85%",
    padding: "14px 18px",
    borderRadius: "16px",
    lineHeight: 1.7,
    fontSize: "15px",
  },
  userMessage: {
    background: "linear-gradient(135deg, #c9a87c 0%, #a08060 100%)",
    color: "#1a1816",
    borderBottomRightRadius: "4px",
    boxShadow: "0 4px 12px rgba(201, 168, 124, 0.2)",
  },
  assistantMessage: {
    background: "rgba(232, 228, 223, 0.06)",
    border: "1px solid rgba(232, 228, 223, 0.1)",
    borderBottomLeftRadius: "4px",
    display: "flex",
    gap: "12px",
  },
  assistantIcon: {
    flexShrink: 0,
    marginTop: "2px",
  },
  messageContent: {
    whiteSpace: "pre-wrap",
    wordBreak: "break-word",
  },
  cursor: {
    display: "inline-block",
    marginLeft: "2px",
    color: "#c9a87c",
    animation: "blink 1s infinite",
  },
  inputArea: {
    padding: "20px 24px",
    borderTop: "1px solid rgba(232, 228, 223, 0.08)",
    background: "rgba(26, 24, 22, 0.9)",
    backdropFilter: "blur(12px)",
  },
  inputWrapper: {
    display: "flex",
    gap: "12px",
    maxWidth: "800px",
    margin: "0 auto",
    background: "rgba(232, 228, 223, 0.04)",
    border: "1px solid rgba(232, 228, 223, 0.1)",
    borderRadius: "16px",
    padding: "8px 8px 8px 16px",
    alignItems: "flex-end",
    transition: "border-color 0.2s ease, box-shadow 0.2s ease",
  },
  textarea: {
    flex: 1,
    background: "transparent",
    border: "none",
    outline: "none",
    color: "#e8e4df",
    fontSize: "15px",
    lineHeight: 1.6,
    resize: "none",
    fontFamily: "'Noto Sans SC', -apple-system, sans-serif",
    padding: "8px 0",
    minHeight: "24px",
    maxHeight: "150px",
  },
  sendButton: {
    width: "44px",
    height: "44px",
    borderRadius: "12px",
    border: "none",
    background: "linear-gradient(135deg, #c9a87c 0%, #a08060 100%)",
    color: "#1a1816",
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    flexShrink: 0,
    transition: "transform 0.2s ease, opacity 0.2s ease",
  },
  sendButtonDisabled: {
    opacity: 0.4,
    cursor: "not-allowed",
    transform: "none",
  },
  loadingDots: {
    display: "flex",
    gap: "2px",
    fontSize: "18px",
  },
  dot: {
    animation: "bounce 0.6s infinite",
  },
  hint: {
    textAlign: "center",
    fontSize: "12px",
    color: "#6a655e",
    margin: "12px 0 0 0",
    maxWidth: "800px",
    marginLeft: "auto",
    marginRight: "auto",
  },
};

export default StreamingChat;