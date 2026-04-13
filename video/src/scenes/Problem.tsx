import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { theme } from "../theme";

const FPS = 30;

const PricingMock: React.FC = () => (
  <div
    style={{
      width: 820,
      background: "white",
      borderRadius: 12,
      color: "#111",
      fontFamily: 'system-ui, -apple-system, "Segoe UI", sans-serif',
      boxShadow: "0 40px 120px rgba(0,0,0,0.6)",
      overflow: "hidden",
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
            background: "#3b82f6",
            color: "white",
            border: "none",
            padding: "14px 32px",
            borderRadius: 8,
            fontSize: 17,
            fontWeight: 600,
          }}
        >
          Get Started
        </button>
      </div>
    </div>
  </div>
);

const formatTimer = (seconds: number): string => {
  const h = Math.floor(seconds / 3600)
    .toString()
    .padStart(2, "0");
  const m = Math.floor((seconds % 3600) / 60)
    .toString()
    .padStart(2, "0");
  const s = Math.floor(seconds % 60)
    .toString()
    .padStart(2, "0");
  return `${h}:${m}:${s}`;
};

export const Problem: React.FC = () => {
  const frame = useCurrentFrame();

  const mockOpacity = interpolate(frame, [0, 20], [0, 1], {
    extrapolateRight: "clamp",
  });

  const bubbleX = interpolate(
    frame,
    [2 * FPS, 2 * FPS + 25],
    [400, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const bubbleOpacity = interpolate(
    frame,
    [2 * FPS, 2 * FPS + 15],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const timerStart = 7 * FPS;
  const timerEnd = 12 * FPS;
  const timerProgress = interpolate(frame, [timerStart, timerEnd], [0, 2400], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const timerOpacity = interpolate(frame, [timerStart, timerStart + 15], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const tagOpacity = interpolate(
    frame,
    [12 * FPS, 12 * FPS + 25],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  return (
    <AbsoluteFill
      style={{
        backgroundColor: theme.bg,
        fontFamily: theme.font,
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div style={{ opacity: mockOpacity }}>
        <PricingMock />
      </div>

      <div
        style={{
          position: "absolute",
          top: 120,
          right: 140,
          transform: `translateX(${bubbleX}px)`,
          opacity: bubbleOpacity,
          background: "#e5e5ea",
          color: "#111",
          padding: "14px 20px",
          borderRadius: 22,
          fontSize: 22,
          fontFamily: 'system-ui, -apple-system, sans-serif',
          maxWidth: 360,
          boxShadow: "0 20px 60px rgba(0,0,0,0.4)",
        }}
      >
        “the button thing is off”
      </div>

      <div
        style={{
          position: "absolute",
          top: 120,
          left: 140,
          opacity: timerOpacity,
          color: theme.muted,
          fontSize: 56,
          fontFeatureSettings: '"tnum"',
          letterSpacing: 2,
        }}
      >
        {formatTimer(timerProgress)}
      </div>

      <div
        style={{
          position: "absolute",
          bottom: 80,
          opacity: tagOpacity,
          color: theme.text,
          fontSize: 48,
          fontWeight: 700,
          letterSpacing: -1,
        }}
      >
        40 minutes. One padding value.
      </div>
    </AbsoluteFill>
  );
};
