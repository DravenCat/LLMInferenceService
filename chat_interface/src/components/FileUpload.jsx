import React, { useState, useRef, forwardRef, useImperativeHandle } from "react";
import styles from "./FileUpload.module.css";

const FileUpload = forwardRef(({
  onFileUploaded,
  onFileRemoved,
  onUploadError,
  disabled,
  attachedFiles = [],
}, ref) => {
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef(null);

  const allowedExtensions = [".txt", ".pdf", ".docx", ".pptx"];

  useImperativeHandle(ref, () => ({
    trigger: () => fileInputRef.current?.click()
  }));

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
    };
    return typeMap[ext] || ext?.toUpperCase() || "File";
  };

  // 获取文件扩展名
  const getFileExt = (filename) => {
    return filename?.split(".").pop().toLowerCase() || "";
  };

  const handleFileSelect = async (e) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // 前端文件类型校验
    const ext = "." + file.name.split(".").pop().toLowerCase();
    if (!allowedExtensions.includes(ext)) {
      onUploadError?.({
        error: "Unsupported file type",
        file_type: ext.slice(1) // 移除前面的点
      });
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
      return;
    }

    setUploading(true);

    try {
      const formData = new FormData();
      formData.append("file", file);

      const response = await fetch("http://localhost:8080/upload", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const errorData = await response.json();
        // 调用父组件的错误处理函数
        onUploadError?.(errorData);
        return;
      }

      const data = await response.json();
      data.filesize = file.size;
      onFileUploaded?.(data);
    } catch (err) {
      // 网络错误等
      onUploadError?.({
        error: "Upload failed",
        file_type: file.name.split('.').pop() || "unknown"
      });
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const handleRemove = async (fileToRemove) => {
    if (!fileToRemove) return;

    try {
      await fetch(`http://localhost:8080/files/${fileToRemove.file_id}`, {
        method: "DELETE",
      });
    } catch (err) {
      console.error("Failed to delete file:", err);
    }

    onFileRemoved?.(fileToRemove.file_id);
  };

  const showStatus = attachedFiles.length > 0 || uploading;

  return (
    <>
      <input
        ref={fileInputRef}
        type="file"
        accept={allowedExtensions.join(",")}
        onChange={handleFileSelect}
        disabled={disabled || uploading}
        className={styles.hiddenInput}
      />

      {showStatus && (
        <div className={styles.fileUploadContainer}>
          {/* 已上传的文件列表 */}
          {attachedFiles.map((file) => (
            <div key={file.file_id} className={styles.fileCard}>
              {/* 文件图标 */}
              <div className={`${styles.fileIcon} ${styles[getFileExt(file.filename)]}`}>
                <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                </svg>
              </div>

              {/* 文件信息 */}
              <div className={styles.fileInfo}>
                <span className={styles.fileName}>{file.filename}</span>
                <span className={styles.fileMeta}>
                  {getFileTypeName(file.filename)} · {formatFileSize(file.filesize)}
                </span>
              </div>

              {/* 删除按钮 */}
              <button
                onClick={() => handleRemove(file)}
                disabled={disabled}
                className={styles.removeButton}
                title="Remove file"
              >
                <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          ))}

          {/* 上传中状态 */}
          {uploading && (
            <div className={styles.uploadingCard}>
              <div className={styles.uploadingIcon}>
                <div className={styles.spinner} />
              </div>
              <div className={styles.uploadingInfo}>
                <span className={styles.title}>Uploading...</span>
                <span className={styles.subtitle}>Please wait</span>
              </div>
            </div>
          )}
        </div>
      )}
    </>
  );
});

FileUpload.displayName = "FileUpload";

export default FileUpload;
