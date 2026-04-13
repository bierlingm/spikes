import React from "react";

type Props = {
  fixedProgress?: number; // 0 = broken (button cut off), 1 = fixed (button fits)
  jiggleFrame?: number;
  width?: number;
};

export const MobilePhone: React.FC<Props> = ({
  fixedProgress = 0,
  jiggleFrame = 0,
  width = 340,
}) => {
  const aspect = 18 / 9;
  const height = width * aspect;
  const jiggle =
    jiggleFrame > 0 && jiggleFrame < 8
      ? Math.sin(jiggleFrame * 1.2) * 2
      : 0;

  // Broken: button positioned at left:90 (overflows right edge, gets clipped).
  // Fixed: button positioned at left:0 with full width inside the card.
  const buttonLeft = 90 * (1 - fixedProgress);
  const buttonStretch = fixedProgress; // 0 = natural width, 1 = fills card

  return (
    <div
      style={{
        width,
        height,
        background: "#0a0a0a",
        borderRadius: 44,
        padding: 10,
        boxShadow: "0 40px 120px rgba(0,0,0,0.7)",
        border: "1px solid #1f1f23",
        transform: `translateX(${jiggle}px)`,
      }}
    >
      <div
        style={{
          width: "100%",
          height: "100%",
          background: "white",
          borderRadius: 36,
          overflow: "hidden",
          position: "relative",
          color: "#111",
          fontFamily: 'system-ui, -apple-system, sans-serif',
        }}
      >
        <div
          style={{
            position: "absolute",
            top: 8,
            left: "50%",
            transform: "translateX(-50%)",
            width: 90,
            height: 22,
            background: "#0a0a0a",
            borderRadius: 12,
            zIndex: 10,
          }}
        />
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            padding: "12px 24px 0",
            fontSize: 11,
            fontWeight: 600,
          }}
        >
          <span>9:41</span>
          <span>5G</span>
        </div>

        <div style={{ padding: "32px 18px 18px" }}>
          <h1 style={{ fontSize: 26, margin: 0, fontWeight: 700 }}>Pricing</h1>
          <div
            style={{
              marginTop: 18,
              border: "1px solid #e5e7eb",
              borderRadius: 10,
              padding: 16,
              overflow: "hidden",
              position: "relative",
            }}
          >
            <h2 style={{ margin: 0, fontSize: 16, fontWeight: 700 }}>Pro Plan</h2>
            <div style={{ fontSize: 26, fontWeight: 700, marginTop: 4 }}>
              $29
              <span style={{ fontSize: 13, fontWeight: 400 }}>/mo</span>
            </div>
            <p style={{ color: "#475569", fontSize: 12, marginTop: 6, marginBottom: 14 }}>
              Everything you need.
            </p>
            <div style={{ position: "relative", height: 42 }}>
              <button
                style={{
                  background: fixedProgress > 0.5 ? "#2563eb" : "#3b82f6",
                  color: "white",
                  border: "none",
                  padding: "10px 24px",
                  borderRadius: 6,
                  fontSize: 14,
                  fontWeight: 700,
                  whiteSpace: "nowrap",
                  position: "absolute",
                  top: 0,
                  left: buttonLeft,
                  right: buttonStretch > 0.95 ? 0 : "auto",
                  width: buttonStretch > 0.95 ? "auto" : "auto",
                  textAlign: "center",
                  display: "block",
                }}
              >
                Get Started Now
              </button>
              {fixedProgress < 0.7 && (
                <div
                  style={{
                    position: "absolute",
                    right: -16,
                    top: -4,
                    bottom: -4,
                    width: 4,
                    background: `linear-gradient(to right, transparent, rgba(231,76,60,${0.6 * (1 - fixedProgress)}))`,
                    pointerEvents: "none",
                  }}
                />
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
