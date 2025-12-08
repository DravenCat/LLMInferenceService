import React, {useEffect, useRef, useState} from "react";


const ModelSelector = ({ model, setModel, models, disabled }) => {
  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef(null);

  useEffect(() => {
    const handleClickOutside = (e) => {
      if (menuRef.current && !menuRef.current.contains(e.target)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const selectedModel = models.find((m) => m.id === model);

  return (
    <div className="relative" ref={menuRef}>
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-stone-700/50 hover:bg-stone-700 text-xs text-stone-400 hover:text-stone-300 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <span>{selectedModel?.name}</span>
        <svg
          className={`w-3 h-3 transition-transform ${isOpen ? "rotate-180" : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 15l7-7 7 7" />
        </svg>
      </button>

      {isOpen && (
        <div className="absolute bottom-full right-0 mb-2 py-1 bg-stone-800 border border-stone-700 rounded-lg shadow-xl min-w-[120px] z-10">
          {models.map((m) => (
            <button
              key={m.id}
              onClick={() => {
                setModel(m.id);
                setIsOpen(false);
              }}
              className={`w-full px-3 py-2 text-left text-sm hover:bg-stone-700 transition-colors ${
                m.id === model ? "text-amber-400" : "text-stone-300"
              }`}
            >
              {m.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};


export default ModelSelector;