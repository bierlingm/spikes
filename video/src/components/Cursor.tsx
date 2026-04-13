import React from "react";

type Props = {
  x: number;
  y: number;
  variant?: "arrow" | "crosshair";
  opacity?: number;
};

export const Cursor: React.FC<Props> = ({
  x,
  y,
  variant = "arrow",
  opacity = 1,
}) => {
  if (variant === "crosshair") {
    const size = 28;
    return (
      <svg
        width={size}
        height={size}
        viewBox="0 0 28 28"
        style={{
          position: "absolute",
          left: x - size / 2,
          top: y - size / 2,
          opacity,
          pointerEvents: "none",
          filter: "drop-shadow(0 2px 6px rgba(0,0,0,0.6))",
        }}
      >
        <line x1="14" y1="2" x2="14" y2="10" stroke="#fafafa" strokeWidth="2" />
        <line x1="14" y1="18" x2="14" y2="26" stroke="#fafafa" strokeWidth="2" />
        <line x1="2" y1="14" x2="10" y2="14" stroke="#fafafa" strokeWidth="2" />
        <line x1="18" y1="14" x2="26" y2="14" stroke="#fafafa" strokeWidth="2" />
        <circle
          cx="14"
          cy="14"
          r="3"
          fill="none"
          stroke="#e74c3c"
          strokeWidth="2"
        />
      </svg>
    );
  }
  return (
    <svg
      width="22"
      height="28"
      viewBox="0 0 22 28"
      style={{
        position: "absolute",
        left: x,
        top: y,
        opacity,
        pointerEvents: "none",
        filter: "drop-shadow(0 2px 6px rgba(0,0,0,0.7))",
      }}
    >
      <path
        d="M1 1 L1 22 L7 17 L11 26 L14 24 L10 16 L18 16 Z"
        fill="#fafafa"
        stroke="#09090b"
        strokeWidth="1.2"
      />
    </svg>
  );
};
