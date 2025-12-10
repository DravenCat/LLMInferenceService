import React, { useState, useRef, useEffect } from "react";
import ModelSelector from "./ModelSelector";

const StreamingChat = () => {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [model, setModel] = useState("llama-3.2-1b-instruct");
  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);
  const abortControllerRef = useRef(null);

  // 后端支持的模型列表
  const models = [
    { id: "llama-3.2-1b-instruct", name: "Llama 3.2 1B" },
    { id: "llama-3.2-3b-instruct", name: "Llama 3.2 3B" },
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

  const handleSubmit = async () => {
    if (!input.trim() || isStreaming) return;

    // 取消之前的请求
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();

    const userMessage = { role: "user", content: input.trim() };
    setMessages((prev) => [...prev, userMessage, { role: "assistant", content: "" }]);
    setInput("");
    setIsStreaming(true);

    try {
      const response = await fetch("http://localhost:8080/generate/stream", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          prompt: userMessage.content,
          model: model,  // 后端使用 "model" 字段
          max_tokens: 512,
          temperature: 0.7,
          top_p: 0.9,
          use_chat_template: true,  // 使用聊天模板，让模型回答问题而非补全
        }),
        signal: abortControllerRef.current.signal,
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Request failed: ${response.status}`);
      }

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
          const trimmedLine = line.trim();
          if (!trimmedLine) continue;

          // 处理 SSE 格式 (data: {...})
          let jsonStr = trimmedLine;
          if (trimmedLine.startsWith("data:")) {
            jsonStr = trimmedLine.slice(5).trim();
            if (jsonStr === "[DONE]" || jsonStr === "keep-alive") continue;
          }

          // 跳过非 JSON 内容
          if (!jsonStr.startsWith("{")) continue;

          try {
            const parsed = JSON.parse(jsonStr);

            // 后端返回格式:
            // { token: "xxx", generated_text: "...", is_finished: bool, finish_reason: "stop"|null }
            const token = parsed.token || "";
            if (token) {
              setMessages((prev) => {
                const newMessages = [...prev];
                const lastMsg = newMessages[newMessages.length - 1];
                newMessages[newMessages.length - 1] = {
                  ...lastMsg,
                  content: lastMsg.content + token,
                };
                return newMessages;
              });
            }

            // 检查是否完成
            if (parsed.is_finished) {
              break;
            }
          } catch (e) {
            // 非JSON格式，忽略
            console.debug("Skip non-JSON line:", jsonStr);
          }
        }
      }
    } catch (error) {
      if (error.name === "AbortError") return;
      console.error("Streaming request error:", error);
      setMessages((prev) => {
        const newMessages = [...prev];
        if (newMessages.length > 0) {
          newMessages[newMessages.length - 1] = {
            ...newMessages[newMessages.length - 1],
            content: `Error: ${error.message || "Something went wrong. Please try again."}`,
          };
        }
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

  const handleClearChat = () => {
    if (!isStreaming) {
      setMessages([]);
    }
  };

  return (
      <div className="flex flex-col h-screen w-full bg-gradient-to-b from-stone-900 to-stone-950 text-stone-200">
        {/* Header */}
        <header className="px-6 py-4 border-b border-stone-800/50 bg-stone-900/80 backdrop-blur-xl">
          <div className="flex items-center justify-between max-w-3xl mx-auto">
            <div className="flex items-center gap-3">
              <div className="w-7 h-7 rounded-full border-2 border-amber-200/60 flex items-center justify-center">
                <div className="w-3 h-3 rounded-full bg-amber-200/60" />
              </div>
              <span className="text-lg font-medium tracking-wide">AI Chat</span>
            </div>
            {messages.length > 0 && (
                <button
                    onClick={handleClearChat}
                    disabled={isStreaming}
                    className="text-sm text-stone-400 hover:text-stone-200 transition-colors disabled:opacity-50"
                >
                  Clear Chat
                </button>
            )}
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
                <p className="text-sm text-stone-500">Type a message to interact with Llama</p>
                <p className="text-xs text-stone-600 mt-4">
                  Current model: <span className="text-amber-400">{models.find(m => m.id === model)?.name}</span>
                </p>
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
                        <div className="whitespace-pre-wrap break-words">
                          {msg.content || (msg.role === "assistant" && isStreaming && idx === messages.length - 1 ? "" : "")}
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
          <div className="flex gap-3 max-w-3xl mx-auto bg-stone-800/40 border border-stone-700/50 rounded-2xl p-2 pl-4 items-end focus-within:border-amber-600/50 focus-within:ring-1 focus-within:ring-amber-600/20 transition-all">
          <textarea
              ref={textareaRef}
              className="flex-1 bg-transparent border-none outline-none text-stone-200 text-[15px] leading-relaxed resize-none py-2 placeholder:text-stone-500 min-h-[24px] max-h-[150px]"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type your message..."
              rows={1}
              disabled={isStreaming}
          />
            <div className="flex items-center gap-2">
              <ModelSelector
                  model={model}
                  setModel={setModel}
                  models={models}
                  disabled={isStreaming}
              />
              <button
                  className={`w-9 h-9 rounded-lg bg-gradient-to-br from-amber-500 to-amber-600 text-stone-900 flex items-center justify-center shrink-0 transition-all hover:scale-105 active:scale-95 ${
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
          <p className="text-center text-xs text-stone-600 mt-3 max-w-3xl mx-auto">
            Press Enter to send, Shift + Enter for new line
          </p>
        </footer>
      </div>
  );
};

export default StreamingChat;