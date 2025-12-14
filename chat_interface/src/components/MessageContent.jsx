import React, { useMemo, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import "katex/dist/katex.min.css";
import styles from "./MessageContent.module.css";

// 修复流式输出中不完整的 Markdown
const fixIncompleteMarkdown = (content) => {
  if (!content) return "";

  let fixed = content;

  // 补全未闭合的代码块 ```
  const codeBlockMatches = fixed.match(/```/g) || [];
  if (codeBlockMatches.length % 2 !== 0) {
    fixed += "\n```";
  }

  // 补全未闭合的行内代码 `（排除 ``` 的情况）
  const withoutCodeBlocks = fixed.replace(/```[\s\S]*?```/g, "");
  const inlineCodeMatches = withoutCodeBlocks.match(/`/g) || [];
  if (inlineCodeMatches.length % 2 !== 0) {
    fixed += "`";
  }

  // 补全未闭合的块级公式 $$
  const blockMathMatches = fixed.match(/\$\$/g) || [];
  if (blockMathMatches.length % 2 !== 0) {
    fixed += "$$";
  }

  // 补全未闭合的行内公式 $（排除 $$ 的情况）
  const withoutBlockMath = fixed.replace(/\$\$[\s\S]*?\$\$/g, "");
  const inlineMathMatches = withoutBlockMath.match(/(?<!\$)\$(?!\$)/g) || [];
  if (inlineMathMatches.length % 2 !== 0) {
    fixed += "$";
  }

  return fixed;
};

// 将裸露的 LaTeX 表达式包裹在 $ 中
const wrapBareLatex = (content) => {
  if (!content) return "";
  
  let result = content;
  
  // 跳过已经在代码块或 $ 中的内容
  // 先标记这些区域
  const codeBlocks = [];
  const mathBlocks = [];
  
  // 提取代码块
  result = result.replace(/```[\s\S]*?```/g, (match) => {
    codeBlocks.push(match);
    return `__CODE_BLOCK_${codeBlocks.length - 1}__`;
  });
  
  // 提取行内代码
  result = result.replace(/`[^`]+`/g, (match) => {
    codeBlocks.push(match);
    return `__CODE_BLOCK_${codeBlocks.length - 1}__`;
  });
  
  // 提取已有的数学公式 $$...$$
  result = result.replace(/\$\$[\s\S]*?\$\$/g, (match) => {
    mathBlocks.push(match);
    return `__MATH_BLOCK_${mathBlocks.length - 1}__`;
  });
  
  // 提取已有的行内公式 $...$
  result = result.replace(/\$[^$]+\$/g, (match) => {
    mathBlocks.push(match);
    return `__MATH_BLOCK_${mathBlocks.length - 1}__`;
  });
  
  // 处理 \[ ... \] 格式（LaTeX display math）
  result = result.replace(/\\\[([\s\S]*?)\\]/g, (match, inner) => {
    return `$$${inner.trim()}$$`;
  });
  
  // 处理 \( ... \) 格式（LaTeX inline math）
  result = result.replace(/\\\(([\s\S]*?)\\\)/g, (match, inner) => {
    return `$${inner.trim()}$`;
  });
  
  // 处理方括号格式 [ 5! = 5 \times 4 ] - 包含 LaTeX 命令的
  result = result.replace(
      /\[\s*([^[\]]*?\\[a-zA-Z]+[^[\]]*?)\s*]/g,
    (match, inner) => `$${inner.trim()}$`
  );
  
  // 处理 \ 5! = 5 \times 4 \times 3 ... \ 格式
  // 匹配以 \ 开头，包含 \times 等命令，以 \ 结尾的表达式
  result = result.replace(
    /\\\s+([^\\]*?(?:\\(?:times|cdot|div|pm|mp|frac|sqrt)[^\\]*?)+)\s*\\/g,
    (match, inner) => `$${inner.trim()}$`
  );
  
  // 处理包含 \times, \cdot 等的独立数学表达式
  // 例如: 5 \times 4 \times 3 = 60
  result = result.replace(
    /(?<![\\$`])(\d+\s*(?:\\(?:times|cdot|div)\s*\d+\s*)+(?:=\s*\d+)?)/g,
    (match, inner) => `$${inner.trim()}$`
  );
  
  // 处理类似 n! 的阶乘表达式后跟 = 和 \times
  result = result.replace(
    /(?<![\\$`])(\w+!\s*=\s*\d+\s*(?:\\(?:times|cdot)\s*\d+\s*)+(?:=\s*\d+)?)/g,
    (match, inner) => `$${inner.trim()}$`
  );
  
  // 恢复数学公式
  mathBlocks.forEach((block, i) => {
    result = result.replace(`__MATH_BLOCK_${i}__`, block);
  });
  
  // 恢复代码块
  codeBlocks.forEach((block, i) => {
    result = result.replace(`__CODE_BLOCK_${i}__`, block);
  });
  
  return result;
};

// 合并处理函数
const processContent = (content, isStreaming) => {
  if (!content) return "";
  
  let processed = content;
  
  // 包裹裸露的 LaTeX
  processed = wrapBareLatex(processed);
  
  // 修复不完整的 Markdown（仅在流式输出时）
  if (isStreaming) {
    processed = fixIncompleteMarkdown(processed);
  }
  
  return processed;
};

// 复制按钮组件
const CopyButton = ({ code }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  return (
    <button className={styles.copyButton} onClick={handleCopy}>
      {copied ? (
        <>
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M5 13l4 4L19 7"
            />
          </svg>
          Copied!
        </>
      ) : (
        <>
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
          </svg>
          Copy
        </>
      )}
    </button>
  );
};

// 代码块组件
const CodeBlock = ({ language, children }) => {
  const code = String(children).replace(/\n$/, "");

  return (
    <div className={styles.codeBlock}>
      <div className={styles.codeHeader}>
        <span className={styles.codeLanguage}>{language || "text"}</span>
        <CopyButton code={code} />
      </div>
      <SyntaxHighlighter
        style={oneDark}
        language={language || "text"}
        PreTag="div"
        className={styles.codeContent}
        customStyle={{
          margin: 0,
          borderRadius: "0 0 0.5rem 0.5rem",
          fontSize: "0.875rem",
        }}
      >
        {code}
      </SyntaxHighlighter>
    </div>
  );
};

// 自定义组件映射
const createComponents = () => ({
  // 代码块和行内代码
  code({ node, inline, className, children, ...props }) {
    const match = /language-(\w+)/.exec(className || "");
    const language = match ? match[1] : "";

    // 代码块
    if (!inline && (language || String(children).includes("\n"))) {
      return <CodeBlock language={language}>{children}</CodeBlock>;
    }

    // 行内代码
    return (
      <code className={styles.inlineCode} {...props}>
        {children}
      </code>
    );
  },

  // 段落
  p({ children }) {
    return <p className={styles.paragraph}>{children}</p>;
  },

  // 标题
  h1({ children }) {
    return <h1 className={styles.heading1}>{children}</h1>;
  },
  h2({ children }) {
    return <h2 className={styles.heading2}>{children}</h2>;
  },
  h3({ children }) {
    return <h3 className={styles.heading3}>{children}</h3>;
  },
  h4({ children }) {
    return <h4 className={styles.heading4}>{children}</h4>;
  },

  // 列表
  ul({ children }) {
    return <ul className={styles.unorderedList}>{children}</ul>;
  },
  ol({ children }) {
    return <ol className={styles.orderedList}>{children}</ol>;
  },
  li({ children }) {
    return <li className={styles.listItem}>{children}</li>;
  },

  // 引用块
  blockquote({ children }) {
    return <blockquote className={styles.blockquote}>{children}</blockquote>;
  },

  // 表格
  table({ children }) {
    return (
      <div className={styles.tableWrapper}>
        <table className={styles.table}>{children}</table>
      </div>
    );
  },
  th({ children }) {
    return <th className={styles.tableHeader}>{children}</th>;
  },
  td({ children }) {
    return <td className={styles.tableCell}>{children}</td>;
  },

  // 链接
  a({ href, children }) {
    return (
      <a
        href={href}
        className={styles.link}
        target="_blank"
        rel="noopener noreferrer"
      >
        {children}
      </a>
    );
  },

  // 水平线
  hr() {
    return <hr className={styles.horizontalRule} />;
  },

  // 强调
  strong({ children }) {
    return <strong className={styles.strong}>{children}</strong>;
  },
  em({ children }) {
    return <em className={styles.emphasis}>{children}</em>;
  },

  // 删除线
  del({ children }) {
    return <del className={styles.strikethrough}>{children}</del>;
  },
});

// 主组件
const MessageContent = ({ content, isStreaming = false, showCursor = false }) => {
  // 处理内容：包裹裸露 LaTeX + 修复不完整 Markdown
  const processedContent = useMemo(() => {
    return processContent(content, isStreaming);
  }, [content, isStreaming]);

  // 缓存组件映射
  const components = useMemo(() => createComponents(), []);

  if (!processedContent) {
    return showCursor ? <span className={styles.cursor}>▊</span> : null;
  }

  return (
    <div className={`${styles.messageContent} ${showCursor ? styles.withCursor : ''}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkMath]}
        rehypePlugins={[
          [
            rehypeKatex,
            {
              throwOnError: false,
              errorColor: "#ef4444",
              strict: false,
            },
          ],
        ]}
        components={components}
      >
        {processedContent}
      </ReactMarkdown>
    </div>
  );
};

export default React.memo(MessageContent);
