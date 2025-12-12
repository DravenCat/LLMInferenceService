import React, { useState, useRef, useEffect, forwardRef, useImperativeHandle } from "react";

const FileUpload = forwardRef(({
                                 onFileUploaded,
                                 disabled,
                                 attachedFile,
                                 onRemove,
                               }, ref) => {
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState(null);
  const fileInputRef = useRef(null);

  const allowedExtensions = [".txt", ".pdf", ".docx", ".pptx"];

  // 暴露 trigger 方法给父组件
  useImperativeHandle(ref, () => ({
    trigger: () => fileInputRef.current?.click()
  }));

  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  const handleFileSelect = async (e) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const ext = "." + file.name.split(".").pop().toLowerCase();
    if (!allowedExtensions.includes(ext)) {
      setError(`不支持的文件类型，支持: ${allowedExtensions.join(", ")}`);
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
        throw new Error(errorData.error || "上传失败");
      }

      const data = await response.json();
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

  const handleRemove = async () => {
    if (!attachedFile) return;

    try {
      await fetch(`http://localhost:8080/files/${attachedFile.file_id}`, {
        method: "DELETE",
      });
    } catch (err) {
      console.error("删除文件失败:", err);
    }

    onRemove?.();
  };

  const getFileIcon = (filename) => {
    const ext = filename?.split(".").pop().toLowerCase();
    const iconMap = {
      pdf: "📄",
      docx: "📝",
      pptx: "📊",
      txt: "📃",
    };
    return iconMap[ext] || "📎";
  };

  // 是否显示状态区域
  const showStatus = attachedFile || uploading || error;

  return (
      <>
        {/* 隐藏的文件输入 */}
        <input
            ref={fileInputRef}
            type="file"
            accept={allowedExtensions.join(",")}
            onChange={handleFileSelect}
            disabled={disabled || uploading}
            className="hidden"
        />

        {/* 状态显示区域 */}
        {showStatus && (
            <div className="px-3 pt-3">
              {/* 已附加的文件 */}
              {attachedFile && (
                  <div className="inline-flex items-center gap-2 px-3 py-2 bg-stone-700/50 rounded-lg text-sm">
                    <span>{getFileIcon(attachedFile.filename)}</span>
                    <span className="text-stone-200 max-w-[200px] truncate">
                {attachedFile.filename}
              </span>
                    <button
                        onClick={handleRemove}
                        disabled={disabled}
                        className="ml-1 p-1 hover:bg-stone-600/50 rounded transition-colors text-stone-400 hover:text-stone-200"
                        title="移除文件"
                    >
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
              )}

              {/* 上传中 */}
              {uploading && (
                  <div className="inline-flex items-center gap-2 px-3 py-2 bg-stone-700/50 rounded-lg text-sm text-stone-400">
                    <div className="w-4 h-4 border-2 border-amber-400 border-t-transparent rounded-full animate-spin" />
                    <span>上传中...</span>
                  </div>
              )}

              {/* 错误 */}
              {error && (
                  <div className="inline-flex items-center gap-2 px-3 py-2 bg-red-900/30 border border-red-700/50 rounded-lg text-sm text-red-300">
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
                      <path strokeLinecap="round" strokeLinejoin="round" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <span>{error}</span>
                  </div>
              )}
            </div>
        )}
      </>
  );
});

FileUpload.displayName = "FileUpload";

export default FileUpload;