import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from "remotion";
import { theme } from "../theme";

const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

export const Title: React.FC = () => {
  const frame = useCurrentFrame();

  // Line 1: in 0-15, holds, fully exits by frame 60.
  const line1Opacity = interpolate(
    frame,
    [0, 15, 50, 60],
    [0, 1, 1, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const line1Lift = interpolate(frame, [50, 70], [0, -14], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Line 2: enters cleanly at 70, lands at 95. No overlap with line 1.
  const line2Opacity = interpolate(frame, [70, 95], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const line2Rise = interpolate(frame, [70, 100], [14, 0], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Sword: arrives last, with anticipation.
  const swordOpacity = interpolate(frame, [105, 120], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const swordX = interpolate(frame, [105, 135], [-40, 0], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const swordScale = interpolate(frame, [105, 125, 140], [0.7, 1.08, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        backgroundColor: theme.bg,
        color: theme.text,
        fontFamily: theme.font,
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div
        style={{
          position: "relative",
          width: 1600,
          height: 200,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        {/* Line 1 — replaced in place */}
        <div
          style={{
            position: "absolute",
            opacity: line1Opacity,
            transform: `translateY(${line1Lift}px)`,
            fontSize: 104,
            fontWeight: 700,
            letterSpacing: -2,
            color: theme.muted,
          }}
        >
          Building is easy now.
        </div>

        {/* Line 2 + sword */}
        <div
          style={{
            position: "absolute",
            opacity: line2Opacity,
            transform: `translateY(${line2Rise}px)`,
            display: "flex",
            alignItems: "center",
            gap: 28,
            fontSize: 104,
            fontWeight: 700,
            letterSpacing: -2,
          }}
        >
          <span
            style={{
              opacity: swordOpacity,
              transform: `translateX(${swordX}px) scale(${swordScale})`,
              transformOrigin: "center",
              color: theme.red,
              display: "inline-block",
              width: 60,
              textAlign: "center",
            }}
          >
            /
          </span>
          <span style={{ color: theme.text }}>
            The feedback loop isn&apos;t.
          </span>
        </div>
      </div>
    </AbsoluteFill>
  );
};
