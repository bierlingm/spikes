import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { theme } from "../theme";

export const Title: React.FC = () => {
  const frame = useCurrentFrame();

  const line1Opacity = interpolate(frame, [0, 20], [0, 1], {
    extrapolateRight: "clamp",
  });
  const line1Shift = interpolate(frame, [50, 70], [0, -40], {
    extrapolateRight: "clamp",
  });

  const line2Clip = interpolate(frame, [60, 85], [0, 100], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const swordX = interpolate(frame, [90, 120], [-80, 0], {
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
          textAlign: "center",
          fontSize: 96,
          fontWeight: 700,
          letterSpacing: -2,
        }}
      >
        <div
          style={{
            opacity: line1Opacity,
            transform: `translateY(${line1Shift}px)`,
          }}
        >
          Building is easy now.
        </div>

        <div
          style={{
            marginTop: 16,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 24,
          }}
        >
          <span
            style={{
              color: theme.red,
              transform: `translateX(${swordX}px)`,
              opacity: swordX === 0 ? 1 : 0.6,
              fontSize: 96,
              fontWeight: 700,
            }}
          >
            /
          </span>
          <span
            style={{
              background: `linear-gradient(90deg, ${theme.red}, ${theme.redDark})`,
              WebkitBackgroundClip: "text",
              WebkitTextFillColor: "transparent",
              backgroundClip: "text",
              clipPath: `inset(0 ${100 - line2Clip}% 0 0)`,
            }}
          >
            The feedback loop isn&apos;t.
          </span>
        </div>
      </div>
    </AbsoluteFill>
  );
};
