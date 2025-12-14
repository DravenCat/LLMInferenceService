import React from "react";
import styles from "./Sidebar.module.css";

const Sidebar = ({
                   isOpen,
                   onToggle,
                   sessions,
                   currentSessionId,
                   onSelectSession,
                   onNewChat,
                   onDeleteSession,
                 }) => {
  // 格式化时间
  const formatTime = (timestamp) => {
    if (!timestamp) return "";

    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString();
  };

  // 获取会话标题（取第一条用户消息的前30个字符）
  const getSessionTitle = (session) => {
    if (session.title) return session.title;
    if (session.firstMessage) {
      return session.firstMessage.length > 30
          ? session.firstMessage.slice(0, 30) + "..."
          : session.firstMessage;
    }
    return "New Chat";
  };

  return (
      <>
        {/* 切换按钮 - 侧边栏打开时隐藏 */}
        {!isOpen && (
            <button className={styles.toggleButton} onClick={onToggle}>
              <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
        )}

        {/* 遮罩层 */}
        <div
            className={`${styles.overlay} ${isOpen ? styles.visible : ""}`}
            onClick={onToggle}
        />

        {/* 侧边栏 */}
        <div className={`${styles.sidebar} ${isOpen ? styles.open : ""}`}>
          {/* 头部 */}
          <div className={styles.sidebarHeader}>
            <div className={styles.headerTitle}>
              <div className={styles.logo}>
                <div className={styles.logoInner} />
              </div>
              <span>Chat</span>
            </div>
            <button className={styles.closeButton} onClick={onToggle}>
              <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* 新建聊天按钮 */}
          <button className={styles.newChatButton} onClick={onNewChat}>
            <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            New Chat
          </button>

          {/* 会话列表 */}
          <div className={styles.sessionListContainer}>
            {sessions.length > 0 ? (
                <>
                  <div className={styles.sessionListTitle}>Recent Chats</div>
                  <div className={styles.sessionList}>
                    {sessions.map((session) => (
                        <div
                            key={session.id}
                            className={`${styles.sessionItem} ${
                                session.id === currentSessionId ? styles.active : ""
                            }`}
                            onClick={() => onSelectSession(session.id)}
                        >
                          <div className={styles.sessionIcon}>
                            <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5">
                              <path strokeLinecap="round" strokeLinejoin="round" d="M8.625 12a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375M21 12c0 4.556-4.03 8.25-9 8.25a9.764 9.764 0 01-2.555-.337A5.972 5.972 0 015.41 20.97a5.969 5.969 0 01-.474-.065 4.48 4.48 0 00.978-2.025c.09-.457-.133-.901-.467-1.226C3.93 16.178 3 14.189 3 12c0-4.556 4.03-8.25 9-8.25s9 3.694 9 8.25z" />
                            </svg>
                          </div>
                          <div className={styles.sessionInfo}>
                      <span className={styles.sessionTitle}>
                        {getSessionTitle(session)}
                      </span>
                            <span className={styles.sessionMeta}>
                        {formatTime(session.updatedAt)}
                      </span>
                          </div>
                          <button
                              className={styles.deleteButton}
                              onClick={(e) => {
                                e.stopPropagation();
                                onDeleteSession(session.id);
                              }}
                          >
                            <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                              <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                            </svg>
                          </button>
                        </div>
                    ))}
                  </div>
                </>
            ) : (
                <div className={styles.emptyState}>
                  <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                  </svg>
                  <p>No chat history yet</p>
                </div>
            )}
          </div>

          {/* 底部 */}
          <div className={styles.sidebarFooter}>
            Powered by Local LLM
          </div>
        </div>
      </>
  );
};

export default Sidebar;