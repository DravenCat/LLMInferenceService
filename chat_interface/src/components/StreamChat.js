import React, { useState, useRef, useEffect } from "react";
import ModelSelector from "./ModelSelector";
import FileUpload from "./FileUpload";
import Sidebar from "./Sidebar";
import styles from "./StreamChat.module.css";

const StreamingChat = () => {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [model, setModel] = useState("qwen");
  const [attachedFiles, setAttachedFiles] = useState([]);
  const [uploadError, setUploadError] = useState(null);
  const [isErrorHiding, setIsErrorHiding] = useState(false);
  
  // 侧边栏和会话管理
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [sessions, setSessions] = useState([]);
  const [currentSessionId, setCurrentSessionId] = useState(null);

  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);
  const fileUploadRef = useRef(null);
  const abortControllerRef = useRef(null);

  const models = [
    { id: "qwen", name: "QWEN" },
    { id: "smollm2", name: "SmolLM2 1.7B" },
    { id: "llama8b", name: "LLaMA 8B" },
  ];

  // 从 localStorage 加载会话列表
  useEffect(() => {
    const savedSessions = localStorage.getItem("chatSessions");
    if (savedSessions) {
      try {
        setSessions(JSON.parse(savedSessions));
      } catch (e) {
        console.error("Failed to parse saved sessions:", e);
      }
    }
  }, []);

  // 保存会话列表到 localStorage
  useEffect(() => {
    if (sessions.length > 0) {
      localStorage.setItem("chatSessions", JSON.stringify(sessions));
    }
  }, [sessions]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 150) + "px";
    }
  }, [input]);

  // 添加文件到列表
  const handleFileUploaded = (fileData) => {
    setAttachedFiles((prev) => [...prev, fileData]);
  };

  // 处理上传错误
  const handleUploadError = (errorData) => {
    setUploadError(errorData);
    setIsErrorHiding(false);
    
    setTimeout(() => {
      setIsErrorHiding(true);
    }, 2700);
    
    setTimeout(() => {
      setUploadError(null);
      setIsErrorHiding(false);
    }, 3000);
  };

  // 从列表中移除文件
  const handleFileRemoved = (fileId) => {
    setAttachedFiles((prev) => prev.filter((f) => f.file_id !== fileId));
  };

  const handlePlusClick = () => {
    fileUploadRef.current?.trigger();
  };

  // 格式化文件大小
  const formatFileSize = (bytes) => {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(unitIndex > 0 ? 1 : 0)} ${units[unitIndex]}`;
  };

  // 获取文件类型显示名称
  const getFileTypeName = (filename) => {
    const ext = filename?.split(".").pop().toLowerCase();
    const typeMap = {
      // 文档
      pdf: "PDF",
      docx: "Word Document",
      pptx: "PowerPoint",
      xlsx: "Excel Spreadsheet",
      txt: "Text File",
      // Markdown
      md: "Markdown",
      markdown: "Markdown",
      // 代码 - Web/脚本
      py: "Python",
      js: "JavaScript",
      ts: "TypeScript",
      jsx: "React JSX",
      tsx: "React TSX",
      vue: "Vue Component",
      svelte: "Svelte",
      // 代码 - 系统/JVM
      rs: "Rust",
      go: "Go",
      java: "Java",
      kt: "Kotlin",
      scala: "Scala",
      // 代码 - C/C++
      c: "C",
      cpp: "C++",
      h: "C Header",
      hpp: "C++ Header",
      // 代码 - .NET
      cs: "C#",
      fs: "F#",
      // 代码 - 动态语言
      rb: "Ruby",
      php: "PHP",
      // 代码 - Apple
      swift: "Swift",
      // Shell
      sh: "Shell Script",
      bash: "Bash Script",
      ps1: "PowerShell",
      // 数据库
      sql: "SQL",
      graphql: "GraphQL",
      // Web 前端
      html: "HTML",
      css: "CSS",
      scss: "SCSS",
      // 配置文件
      json: "JSON",
      yaml: "YAML",
      yml: "YAML",
      toml: "TOML",
      xml: "XML",
      ini: "INI Config",
      // 其他
      log: "Log File",
      env: "Environment",
      dockerfile: "Dockerfile",
    };
    return typeMap[ext] || ext?.toUpperCase() || "File";
  };

  // 获取文件扩展名
  const getFileExt = (filename) => {
    return filename?.split(".").pop().toLowerCase() || "";
  };

  // 更新会话
  const updateSession = (sessionId, newMessages, firstMessage = null) => {
    setSessions((prev) => {
      const existingIndex = prev.findIndex((s) => s.id === sessionId);
      const now = new Date().toISOString();
      
      if (existingIndex >= 0) {
        // 更新现有会话
        const updated = [...prev];
        updated[existingIndex] = {
          ...updated[existingIndex],
          messages: newMessages,
          updatedAt: now,
        };
        // 将更新的会话移到最前面
        const [session] = updated.splice(existingIndex, 1);
        return [session, ...updated];
      } else {
        // 创建新会话
        const newSession = {
          id: sessionId,
          messages: newMessages,
          firstMessage: firstMessage || newMessages[0]?.content || "New Chat",
          createdAt: now,
          updatedAt: now,
        };
        return [newSession, ...prev];
      }
    });
  };

  const handleSubmit = async () => {
    if (!input.trim() || isStreaming) return;

    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    // 构建用户消息
    const userMessage = {
      role: "user",
      content: input.trim(),
      files: attachedFiles.length > 0 ? [...attachedFiles] : null
    };
    
    const newMessages = [...messages, userMessage, { role: "assistant", content: "" }];
    setMessages(newMessages);

    const currentPrompt = input.trim();
    const isFirstMessage = messages.length === 0;

    setInput("");
    setAttachedFiles([]);
    setIsStreaming(true);

    try {
      const requestBody = {
        prompt: currentPrompt,
        model_name: model,
      };

      // 如果已有会话 ID，传给后端
      if (currentSessionId) {
        requestBody.session_id = currentSessionId;
      }

      const response = await fetch("http://localhost:8080/generate/stream", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(requestBody),
        signal: abortControllerRef.current.signal,
      });

      if (!response.ok) throw new Error("Request failed");

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      let fullResponse = "";
      let receivedSessionId = currentSessionId;

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";

        for (const line of lines) {
          // 处理会话信息事件
          if (line.startsWith("event: session")) {
            continue;
          }
          
          if (line.startsWith("data: ")) {
            const data = line.slice(6).trim();
            if (data === "[DONE]") continue;

            try {
              const parsed = JSON.parse(data);
              
              // 检查是否是会话信息
              if (parsed.session_id && parsed.type === "session_info") {
                receivedSessionId = parsed.session_id;
                if (!currentSessionId) {
                  setCurrentSessionId(receivedSessionId);
                }
                continue;
              }
              
              const content = parsed.content || "";
              if (content) {
                fullResponse += content;
                setMessages((prev) => {
                  const updated = [...prev];
                  updated[updated.length - 1] = {
                    ...updated[updated.length - 1],
                    content: updated[updated.length - 1].content + content,
                  };
                  return updated;
                });
              }
            } catch {
              // ignore parse errors
            }
          }
        }
      }

      // 推理完成后更新会话
      if (receivedSessionId) {
        const finalMessages = [
          ...messages,
          userMessage,
          { role: "assistant", content: fullResponse }
        ];
        updateSession(
          receivedSessionId,
          finalMessages,
          isFirstMessage ? currentPrompt : null
        );
        if (!currentSessionId) {
          setCurrentSessionId(receivedSessionId);
        }
      }

    } catch (error) {
      if (error.name === "AbortError") return;
      console.error("Streaming request error:", error);
      setMessages((prev) => {
        const updated = [...prev];
        updated[updated.length - 1] = {
          ...updated[updated.length - 1],
          content: "Sorry, something went wrong. Please try again.",
        };
        return updated;
      });
    } finally {
      setIsStreaming(false);
      abortControllerRef.current = null;
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  // 侧边栏操作
  const handleToggleSidebar = () => {
    setIsSidebarOpen((prev) => !prev);
  };

  const handleNewChat = () => {
    setMessages([]);
    setCurrentSessionId(null);
    setAttachedFiles([]);
    setInput("");
    setIsSidebarOpen(false);
  };

  const handleSelectSession = (sessionId) => {
    const session = sessions.find((s) => s.id === sessionId);
    if (session) {
      setMessages(session.messages || []);
      setCurrentSessionId(sessionId);
      setAttachedFiles([]);
      setInput("");
    }
    setIsSidebarOpen(false);
  };

  const handleDeleteSession = async (sessionId) => {
    // 调用后端删除会话
    try {
      await fetch(`http://localhost:8080/session/${sessionId}`, {
        method: "DELETE",
      });
    } catch (e) {
      console.error("Failed to delete session from server:", e);
    }

    // 从本地状态删除
    setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    
    // 如果删除的是当前会话，清空聊天
    if (sessionId === currentSessionId) {
      setMessages([]);
      setCurrentSessionId(null);
    }

    // 更新 localStorage
    const updatedSessions = sessions.filter((s) => s.id !== sessionId);
    localStorage.setItem("chatSessions", JSON.stringify(updatedSessions));
  };

  // 渲染消息中的文件卡片
  const renderFileCard = (file) => {
    const ext = getFileExt(file.filename);
    return (
      <div key={file.file_id} className={styles.messageFileCard}>
        <div className={`${styles.fileIcon} ${styles[ext]}`}>
          <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5">
            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
          </svg>
        </div>
        <div className={styles.fileInfo}>
          <span className={styles.fileName}>{file.filename}</span>
          <span className={styles.fileMeta}>
            {getFileTypeName(file.filename)} · {formatFileSize(file.filesize)}
          </span>
        </div>
      </div>
    );
  };

  return (
    <div className={styles.chatContainer}>
      {/* Sidebar */}
      <Sidebar
        isOpen={isSidebarOpen}
        onToggle={handleToggleSidebar}
        sessions={sessions}
        currentSessionId={currentSessionId}
        onSelectSession={handleSelectSession}
        onNewChat={handleNewChat}
        onDeleteSession={handleDeleteSession}
      />

      {/* Error Toast */}
      {uploadError && (
        <div className={`${styles.errorToast} ${isErrorHiding ? styles.hiding : ''}`}>
          <div className={styles.errorToastIcon}>
            <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
            </svg>
          </div>
          <div className={styles.errorToastContent}>
            <span className={styles.errorToastTitle}>{uploadError.error}</span>
            <span className={styles.errorToastMessage}>The selected file format is not supported</span>
            <span className={styles.errorToastFileType}>
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
              </svg>
              .{uploadError.file_type}
            </span>
          </div>
        </div>
      )}

      {/* Header - 添加左边距给侧边栏按钮留空间 */}
      <header className={styles.header}>
        <div className={styles.headerContent}>
          <div className={styles.headerSpacer} />
          <div className={styles.logo}>
            <div className={styles.logoInner} />
          </div>
          <span className={styles.title}>Chat</span>
        </div>
      </header>

      {/* Messages */}
      <main className={styles.messagesArea}>
        {messages.length === 0 ? (
          <div className={styles.emptyState}>
            <svg className={styles.emptyIcon} fill="none" viewBox="0 0 48 48" stroke="currentColor" strokeWidth="1.5">
              <path strokeLinejoin="round" d="M24 4L28.5 15.5L40 20L28.5 24.5L24 36L19.5 24.5L8 20L19.5 15.5L24 4Z" />
            </svg>
            <p className={styles.emptyTitle}>Start New Chat</p>
            <p className={styles.emptySubtitle}>Type a message or attach a file to start</p>
          </div>
        ) : (
          <div className={styles.messagesContainer}>
            {messages.map((msg, idx) => (
              <div
                key={idx}
                className={`${styles.messageWrapper} ${styles[msg.role]}`}
              >
                {msg.role === "user" ? (
                  <div className={styles.userMessageContainer}>
                    {msg.files && msg.files.length > 0 && (
                      <div className={styles.userFiles}>
                        {msg.files.map(renderFileCard)}
                      </div>
                    )}
                    <div className={styles.userBubble}>
                      <div className={styles.content}>{msg.content}</div>
                    </div>
                  </div>
                ) : (
                  <div className={styles.assistantBubble}>
                    <div className={styles.avatar}>
                      <div className={styles.avatarInner} />
                    </div>
                    <div className={styles.content}>
                      {msg.content}
                      {isStreaming && idx === messages.length - 1 && (
                        <span className={styles.cursor}>▊</span>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}
      </main>

      {/* Input */}
      <footer className={styles.footer}>
        <div className={styles.footerContent}>
          <div className={styles.inputContainer}>
            <FileUpload
              ref={fileUploadRef}
              onFileUploaded={handleFileUploaded}
              onFileRemoved={handleFileRemoved}
              onUploadError={handleUploadError}
              disabled={isStreaming}
              attachedFiles={attachedFiles}
            />

            <div className={styles.inputRow}>
              <button
                onClick={handlePlusClick}
                disabled={isStreaming}
                className={styles.attachButton}
                title="Upload File (txt, pdf, docx, pptx)"
              >
                <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
                </svg>
              </button>

              <textarea
                ref={textareaRef}
                className={styles.textarea}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Send a message..."
                rows={1}
                disabled={isStreaming}
              />

              <div className={styles.inputActions}>
                <ModelSelector
                  model={model}
                  setModel={setModel}
                  models={models}
                  disabled={isStreaming}
                />
                <button
                  className={styles.sendButton}
                  onClick={handleSubmit}
                  disabled={isStreaming || !input.trim()}
                >
                  {isStreaming ? (
                    <div className={styles.loadingDots}>
                      <span className={styles.dot} />
                      <span className={styles.dot} />
                      <span className={styles.dot} />
                    </div>
                  ) : (
                    <svg fill="none" viewBox="0 0 20 20" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M3 10H17M17 10L12 5M17 10L12 15" />
                    </svg>
                  )}
                </button>
              </div>
            </div>
          </div>

          <p className={styles.footerHint}>
            Enter to send · Shift + Enter for new line · Supports documents, code files, and more
          </p>
        </div>
      </footer>
    </div>
  );
};

export default StreamingChat;
