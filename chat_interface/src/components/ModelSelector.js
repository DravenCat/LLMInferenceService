import React, { useEffect, useRef, useState } from "react";
import styles from "./ModelSelector.module.css";

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
    <div className={styles.container} ref={menuRef}>
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={styles.trigger}
      >
        <span>{selectedModel?.name}</span>
        <svg
          className={`${styles.triggerIcon} ${isOpen ? styles.open : ""}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M5 15l7-7 7 7" />
        </svg>
      </button>

      {isOpen && (
        <div className={styles.dropdown}>
          {models.map((m) => (
            <button
              key={m.id}
              onClick={() => {
                setModel(m.id);
                setIsOpen(false);
              }}
              className={`${styles.option} ${m.id === model ? styles.selected : ""}`}
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
