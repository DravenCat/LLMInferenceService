import React, { useState, useRef, useEffect, forwardRef, useImperativeHandle } from "react";

const FileUpload = forwardRef(({
                                 onFileUploaded,
                                 onFileRemoved,
                                 disabled,
                                 attachedFiles = [],
                               }, ref) => {
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState(null);
  const fileInputRef = useRef(null);

  const allowedExtensions = [".txt", ".pdf", ".docx"];

  useImperativeHandle(ref, () => ({
    trigger: () => fileInputRef.current?.click()
  }));

  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [error]);

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

  // 获取文件图标（根据类型显示不同颜色）
  const getFileIcon = (filename) => {
    const ext = filename?.split(".").pop().toLowerCase();

    const iconColors = {
      pdf: "text-red-400",
      docx: "text-blue-400",
      txt: "text-stone-400",
    };

    const colorClass = iconColors[ext] || "text-stone-400";

    return (
        <div className={`w-10 h-10 rounded-lg bg-stone-700/80 flex items-center justify-center ${colorClass}`}>
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5">
            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
          </svg>
        </div>
    );
  };

  const handleFileSelect = async (e) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const ext = "." + file.name.split(".").pop().toLowerCase();
    if (!allowedExtensions.includes(ext)) {
      setError(`Unsupported file type. Please upload: ${allowedExtensions.join(", ")}`);
      return;
    }

    setError(null);
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
        throw new Error(errorData.error || "Failed to upload");
      }

      const data = await response.json();
      // 添加文件大小信息
      data.filesize = file.size;
      onFileUploaded?.(data);
    } catch (err) {
      setError(err.message);
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

  const showStatus = attachedFiles.length > 0 || uploading || error;

  return (
      <>
        <input
            ref={fileInputRef}
            type="file"
            accept={allowedExtensions.join(",")}
            onChange={handleFileSelect}
            disabled={disabled || uploading}
            className="hidden"
        />

        {showStatus && (
            <div className="px-3 pt-3 flex flex-wrap gap-2 justify-start items-start">
              {/* 已上传的文件列表 - Claude 风格卡片 */}
              {attachedFiles.map((file) => (
                  <div
                      key={file.file_id}
                      className="inline-flex items-center gap-3 p-2 pr-3 bg-stone-800/60 border border-stone-700/50 rounded-xl hover:bg-stone-800/80 transition-colors group"
                  >
                    {/* 文件图标 */}
                    {getFileIcon(file.filename)}

                    {/* 文件信息 */}
                    <div className="flex flex-col min-w-0">
                <span className="text-sm text-stone-200 font-medium truncate max-w-[180px]">
                  {file.filename}
                </span>
                      <span className="text-xs text-stone-500">
                  {getFileTypeName(file.filename)} · {formatFileSize(file.filesize)}
                </span>
                    </div>

                    {/* 删除按钮 */}
                    <button
                        onClick={() => handleRemove(file)}
                        disabled={disabled}
                        className="ml-1 p-1.5 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-stone-600/50 transition-all text-stone-400 hover:text-stone-200"
                        title="Remove file"
                    >
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
              ))}

              {/* 上传中状态 */}
              {uploading && (
                  <div className="inline-flex items-center gap-3 p-2 pr-4 bg-stone-800/60 border border-stone-700/50 rounded-xl">
                    <div className="w-10 h-10 rounded-lg bg-stone-700/80 flex items-center justify-center">
                      <div className="w-5 h-5 border-2 border-amber-400 border-t-transparent rounded-full animate-spin" />
                    </div>
                    <div className="flex flex-col">
                      <span className="text-sm text-stone-200">Uploading...</span>
                      <span className="text-xs text-stone-500">Please wait</span>
                    </div>
                  </div>
              )}

              {/* 错误状态 */}
              {error && (
                  <div className="inline-flex items-center gap-3 p-2 pr-4 bg-red-900/20 border border-red-800/50 rounded-xl">
                    <div className="w-10 h-10 rounded-lg bg-red-900/30 flex items-center justify-center text-red-400">
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                      </svg>
                    </div>
                    <div className="flex flex-col">
                      <span className="text-sm text-red-300">Upload failed</span>
                      <span className="text-xs text-red-400/70">{error}</span>
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