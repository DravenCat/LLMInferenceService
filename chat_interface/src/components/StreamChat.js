import React, { useState, useRef, useEffect } from "react";
import ModelSelector from "./ModelSelector";
import FileUpload from "./FileUpload";

const StreamingChat = () => {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [model, setModel] = useState("qwen");
  const [attachedFile, setAttachedFile] = useState(null);

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

  const handleFileUploaded = (fileData) => {
    setAttachedFile(fileData);
  };

  const handleRemoveFile = () => {
    setAttachedFile(null);
  };

  const handlePlusClick = () => {
    fileUploadRef.current?.trigger();
  };

  const handleSubmit = async () => {
    if (!input.trim() || isStreaming) return;

    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    const userMessageContent = attachedFile
        ? `📎 ${attachedFile.filename}\n\n${input.trim()}`
        : input.trim();

    const userMessage = { role: "user", content: userMessageContent };
    setMessages((prev) => [...prev, userMessage, { role: "assistant", content: "" }]);

    const currentPrompt = input.trim();
    const currentFileId = attachedFile?.file_id;

    setInput("");
    setAttachedFile(null);
    setIsStreaming(true);

    try {
      const requestBody = {
        prompt: currentPrompt,
        model_name: model,
      };

      if (currentFileId) {
        requestBody.file_id = currentFileId;
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

  return (
      <div className="flex flex-col h-screen w-full bg-gradient-to-b from-stone-900 to-stone-950 text-stone-200">
        {/* Header */}
        <header className="px-6 py-4 border-b border-stone-800/50 bg-stone-900/80 backdrop-blur-xl">
          <div className="flex items-center gap-3 max-w-3xl mx-auto">
            <div className="w-7 h-7 rounded-full border-2 border-amber-200/60 flex items-center justify-center">
              <div className="w-3 h-3 rounded-full bg-amber-200/60" />
            </div>
            <span className="text-lg font-medium tracking-wide">Chat</span>
          </div>
        </header>

        {/* Messages */}
        <main className="flex-1 overflow-y-auto p-6 scroll-smooth">
          {messages.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full opacity-50">
                <svg className="w-12 h-12 mb-4 text-stone-500" fill="none" viewBox="0 0 48 48" stroke="currentColor" strokeWidth="1.5">
                  <path strokeLinejoin="round" d="M24 4L28.5 15.5L40 20L28.5 24.5L24 36L19.5 24.5L8 20L19.5 15.5L24 4Z" />
                </svg>
                <p className="text-xl mb-2">Start New Chat</p>
                <p className="text-sm text-stone-500">Type a message or attach a file to start</p>
              </div>
          ) : (
              <div className="max-w-3xl mx-auto space-y-5">
                {messages.map((msg, idx) => (
                    <div
                        key={idx}
                        className={`flex animate-in fade-in slide-in-from-bottom-2 duration-300 ${
                            msg.role === "user" ? "justify-end" : "justify-start"
                        }`}
                    >
                      <div
                          className={`max-w-[85%] px-4 py-3 rounded-2xl leading-relaxed ${
                              msg.role === "user"
                                  ? "bg-gradient-to-br from-amber-600 to-amber-700 text-stone-900 rounded-br-sm shadow-lg shadow-amber-900/20"
                                  : "bg-stone-800/50 border border-stone-700/50 rounded-bl-sm flex gap-3"
                          }`}
                      >
                        {msg.role === "assistant" && (
                            <div className="w-4 h-4 mt-0.5 rounded-full border-[1.5px] border-amber-400/70 flex items-center justify-center shrink-0">
                              <div className="w-1.5 h-1.5 rounded-full bg-amber-400/70" />
                            </div>
                        )}
                        <div className="whitespace-pre-wrap break-words text-left">
                          {msg.content}
                          {msg.role === "assistant" && isStreaming && idx === messages.length - 1 && (
                              <span className="inline-block ml-0.5 text-amber-400 animate-pulse">▊</span>
                          )}
                        </div>
                      </div>
                    </div>
                ))}
                <div ref={messagesEndRef} />
              </div>
          )}
        </main>

        {/* Input */}
        <footer className="p-5 border-t border-stone-800/50 bg-stone-900/90 backdrop-blur-xl">
          <div className="max-w-3xl mx-auto">
            <div className="bg-stone-800/40 border border-stone-700/50 rounded-2xl focus-within:border-amber-600/50 focus-within:ring-1 focus-within:ring-amber-600/20 transition-all">

              {/* FileUpload 组件 - 只渲染状态区域和隐藏的 input */}
              <FileUpload
                  ref={fileUploadRef}
                  onFileUploaded={handleFileUploaded}
                  disabled={isStreaming}
                  attachedFile={attachedFile}
                  onRemove={handleRemoveFile}
              />

              {/* 输入行 */}
              <div className="flex items-end gap-1 p-2">
                {/* 回形针按钮 */}
                <button
                    onClick={handlePlusClick}
                    disabled={isStreaming}
                    className={`w-9 h-9 rounded-full flex items-center justify-center shrink-0 transition-all
                  ${isStreaming
                        ? "text-stone-600 cursor-not-allowed"
                        : "text-stone-400 hover:text-stone-200 hover:bg-stone-700/50"
                    }`}
                    title="上传文件 (txt, pdf, docx, pptx)"
                >
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
                  </svg>
                </button>

                {/* 文本输入 */}
                <textarea
                    ref={textareaRef}
                    className="flex-1 bg-transparent border-none outline-none text-stone-200 text-[15px] leading-relaxed resize-none py-2 px-2 placeholder:text-stone-500 min-h-[24px] max-h-[150px]"
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Send a message..."
                    rows={1}
                    disabled={isStreaming}
                />

                {/* 模型选择和发送按钮 */}
                <div className="flex items-center gap-2">
                  <ModelSelector
                      model={model}
                      setModel={setModel}
                      models={models}
                      disabled={isStreaming}
                  />
                  <button
                      className={`w-9 h-9 rounded-full bg-gradient-to-br from-amber-500 to-amber-600 text-stone-900 flex items-center justify-center shrink-0 transition-all hover:scale-105 active:scale-95 ${
                          isStreaming || !input.trim() ? "opacity-40 cursor-not-allowed hover:scale-100" : ""
                      }`}
                      onClick={handleSubmit}
                      disabled={isStreaming || !input.trim()}
                  >
                    {isStreaming ? (
                        <div className="flex gap-0.5">
                          {[0, 1, 2].map((i) => (
                              <span key={i} className="w-1 h-1 bg-stone-900 rounded-full animate-bounce" style={{ animationDelay: `${i * 150}ms` }} />
                          ))}
                        </div>
                    ) : (
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 20 20" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M3 10H17M17 10L12 5M17 10L12 15" />
                        </svg>
                    )}
                  </button>
                </div>
              </div>
            </div>

            <p className="text-center text-xs text-stone-600 mt-3">
              Enter to send · Shift + Enter for new line · Supports txt, pdf, docx, pptx
            </p>
          </div>
        </footer>
      </div>
  );
};

export default StreamingChat;