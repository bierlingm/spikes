import React from "react";

type Props = {
  highlightCta?: boolean;
  brighterCta?: boolean;
  pulsePin?: boolean;
  width?: number;
};

export const PricingMock: React.FC<Props> = ({
  highlightCta = false,
  brighterCta = false,
  pulsePin = false,
  width = 820,
}) => (
  <div
    style={{
      width,
      background: "white",
      borderRadius: 12,
      color: "#111",
      fontFamily: 'system-ui, -apple-system, "Segoe UI", sans-serif',
      boxShadow: "0 40px 120px rgba(0,0,0,0.6)",
      overflow: "hidden",
      position: "relative",
    }}
  >
    <div
      style={{
        background: "#f3f4f6",
        padding: "12px 20px",
        display: "flex",
        gap: 8,
        borderBottom: "1px solid #e5e7eb",
      }}
    >
      {["#fb7185", "#fbbf24", "#4ade80"].map((c) => (
        <span
          key={c}
          style={{
            width: 14,
            height: 14,
            borderRadius: "50%",
            background: c,
          }}
        />
      ))}
    </div>
    <div style={{ padding: "48px 60px" }}>
      <h1 style={{ fontSize: 40, fontWeight: 700, margin: 0 }}>Pricing</h1>
      <div
        style={{
          marginTop: 32,
          border: "2px solid #e5e7eb",
          borderRadius: 12,
          padding: 32,
        }}
      >
        <h2 style={{ margin: 0, fontSize: 24 }}>Pro Plan</h2>
        <div style={{ fontSize: 42, fontWeight: 700, marginTop: 8 }}>
          $29<span style={{ fontSize: 18, fontWeight: 400 }}>/mo</span>
        </div>
        <p style={{ color: "#475569", marginTop: 8 }}>
          Everything you need to get started.
        </p>
        <button
          style={{
            marginTop: 24,
            background: brighterCta ? "#2563eb" : "#3b82f6",
            color: "white",
            border: "none",
            padding: brighterCta ? "16px 36px" : "14px 32px",
            borderRadius: 8,
            fontSize: brighterCta ? 18 : 17,
            fontWeight: 700,
            outline: highlightCta ? "3px solid #e74c3c" : "none",
            outlineOffset: 3,
            boxShadow: highlightCta
              ? "0 0 0 6px rgba(231,76,60,0.18)"
              : "none",
          }}
        >
          Get Started
        </button>
      </div>
    </div>
    {pulsePin && (
      <div
        style={{
          position: "absolute",
          right: 20,
          bottom: 20,
          width: 44,
          height: 44,
          borderRadius: "50%",
          background: "#09090b",
          color: "#fafafa",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: 22,
          fontWeight: 700,
          boxShadow: "0 0 0 0 rgba(231,76,60,0.6)",
          animation: "pulse 2s infinite",
        }}
      >
        /
      </div>
    )}
  </div>
);
