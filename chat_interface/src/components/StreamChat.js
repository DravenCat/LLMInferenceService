import React, { useState, useRef, useEffect } from "react";
import ModelSelector from "./ModelSelector";
import FileUpload from "./FileUpload";
import styles from "./StreamChat.module.css";

const StreamingChat = () => {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [model, setModel] = useState("qwen");
  const [attachedFiles, setAttachedFiles] = useState([]);

  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);
  const fileUploadRef = useRef(null);
  const abortControllerRef = useRef(null);

  const models = [
    { id: "qwen", name: "QWEN" },
    { id: "smollm2", name: "SmolLM2 1.7B" },
    { id: "llama8b", name: "LLaMA 8B" },
  ];

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
      pdf: "PDF",
      docx: "Word Document",
      txt: "Text File",
      pptx: "Power Point File"
    };
    return typeMap[ext] || ext?.toUpperCase() || "File";
  };

  // 获取文件扩展名
  const getFileExt = (filename) => {
    return filename?.split(".").pop().toLowerCase() || "";
  };

  const handleSubmit = async () => {
    if (!input.trim() || isStreaming) return;

    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    // 构建用户消息，包含文件信息
    const userMessage = {
      role: "user",
      content: input.trim(),
      files: attachedFiles.length > 0 ? [...attachedFiles] : null
    };
    setMessages((prev) => [...prev, userMessage, { role: "assistant", content: "" }]);

    const currentPrompt = input.trim();
    const currentFileIds = attachedFiles.map((f) => f.file_id);

    setInput("");
    setAttachedFiles([]);
    setIsStreaming(true);

    try {
      const requestBody = {
        prompt: currentPrompt,
        model_name: model,
      };

      // 支持多个文件 ID
      if (currentFileIds.length > 0) {
        requestBody.file_ids = currentFileIds;
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

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";

        for (const line of lines) {
          if (line.startsWith("data: ")) {
            const data = line.slice(6).trim();
            if (data === "[DONE]") continue;

            try {
              const parsed = JSON.parse(data);
              const content = parsed.content || "";
              if (content) {
                setMessages((prev) => {
                  const newMessages = [...prev];
                  newMessages[newMessages.length - 1] = {
                    ...newMessages[newMessages.length - 1],
                    content: newMessages[newMessages.length - 1].content + content,
                  };
                  return newMessages;
                });
              }
            } catch {
              // ignore
            }
          }
        }
      }
    } catch (error) {
      if (error.name === "AbortError") return;
      console.error("Streaming request error:", error);
      setMessages((prev) => {
        const newMessages = [...prev];
        newMessages[newMessages.length - 1] = {
          ...newMessages[newMessages.length - 1],
          content: "Sorry, something went wrong. Please try again.",
        };
        return newMessages;
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
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.headerContent}>
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
                  // 用户消息 - 文件卡片在气泡上方
                  <div className={styles.userMessageContainer}>
                    {/* 文件卡片区域 */}
                    {msg.files && msg.files.length > 0 && (
                      <div className={styles.userFiles}>
                        {msg.files.map(renderFileCard)}
                      </div>
                    )}
                    {/* 文字气泡 */}
                    <div className={styles.userBubble}>
                      <div className={styles.content}>{msg.content}</div>
                    </div>
                  </div>
                ) : (
                  // 助手消息
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
            {/* FileUpload 组件 */}
            <FileUpload
              ref={fileUploadRef}
              onFileUploaded={handleFileUploaded}
              onFileRemoved={handleFileRemoved}
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
            Enter to send · Shift + Enter for new line · Supports txt, pdf, docx, pptx
          </p>
        </div>
      </footer>
    </div>
  );
};

export default StreamingChat;
