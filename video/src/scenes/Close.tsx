import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from "remotion";
import { theme } from "../theme";
import { MobilePhone } from "../components/MobilePhone";

const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

type PinProps = { x: number; y: number; delay: number; frame: number };

const FloatingPin: React.FC<PinProps> = ({ x, y, delay, frame }) => {
  const local = frame - delay;
  const opacity = interpolate(local, [0, 12, 55, 80], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const translateY = interpolate(local, [0, 80], [0, -160], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        transform: `translateY(${translateY}px)`,
        opacity,
        width: 44,
        height: 44,
        borderRadius: 22,
        background: theme.green,
        color: theme.bg,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        fontSize: 24,
        fontWeight: 800,
        boxShadow: "0 12px 36px rgba(34,197,94,0.4)",
      }}
    >
      ✓
    </div>
  );
};

export const Close: React.FC = () => {
  const frame = useCurrentFrame();

  // Phone fades in 0-15
  const phoneIn = interpolate(frame, [0, 15], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Button morphs from broken (0) to fixed (1) between frames 30-70.
  const fixedProgress = interpolate(frame, [30, 70], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Phone fades out 130-160 to surrender stage
  const phoneOut = interpolate(frame, [130, 160], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // "FIXED" tag pops on phone at 70
  const fixedTagOpacity = interpolate(frame, [70, 95], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const fixedTagScale = interpolate(frame, [70, 95, 110], [0.6, 1.12, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Mantra at 150
  const mantraOpacity = interpolate(frame, [150, 195], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const mantraRise = interpolate(frame, [150, 195], [16, 0], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Slash + URL at 220
  const brandOpacity = interpolate(frame, [220, 255], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const slashScale = interpolate(frame, [220, 250, 270], [0.5, 1.15, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const urlScale = interpolate(frame, [235, 265, 285], [0.85, 1.06, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill style={{ backgroundColor: theme.bg, fontFamily: theme.font }}>
      {/* Phone — center stage with fix morph + floating pins */}
      <AbsoluteFill
        style={{
          alignItems: "center",
          justifyContent: "center",
          opacity: phoneIn * phoneOut,
        }}
      >
        <div style={{ position: "relative" }}>
          <MobilePhone fixedProgress={fixedProgress} width={360} />

          {/* FIXED tag */}
          <div
            style={{
              position: "absolute",
              top: -28,
              right: -36,
              opacity: fixedTagOpacity,
              transform: `scale(${fixedTagScale})`,
              background: theme.green,
              color: theme.bg,
              padding: "8px 16px",
              borderRadius: 999,
              fontSize: 20,
              fontWeight: 800,
              letterSpacing: 1,
              boxShadow: "0 12px 30px rgba(34,197,94,0.4)",
            }}
          >
            ✓ FIXED
          </div>

          {/* Floating resolved pins around the phone */}
          <FloatingPin x={-100} y={120} delay={75} frame={frame} />
          <FloatingPin x={400} y={80} delay={88} frame={frame} />
          <FloatingPin x={-60} y={500} delay={101} frame={frame} />
        </div>
      </AbsoluteFill>

      {/* End card */}
      <AbsoluteFill
        style={{
          alignItems: "center",
          justifyContent: "center",
          gap: 56,
        }}
      >
        <div
          style={{
            opacity: mantraOpacity,
            transform: `translateY(${mantraRise}px)`,
            color: theme.text,
            fontSize: 96,
            fontWeight: 700,
            letterSpacing: -2,
          }}
        >
          Build. Spike. Fix. Repeat.
        </div>

        {/* Slash brand mark + URL */}
        <div
          style={{
            opacity: brandOpacity,
            display: "flex",
            alignItems: "center",
            gap: 30,
          }}
        >
          <span
            style={{
              color: theme.red,
              fontSize: 140,
              fontWeight: 800,
              lineHeight: 1,
              transform: `scale(${slashScale})`,
              transformOrigin: "center",
              display: "inline-block",
            }}
          >
            /
          </span>
          <span
            style={{
              color: theme.text,
              fontSize: 110,
              fontWeight: 800,
              letterSpacing: -2,
              transform: `scale(${urlScale})`,
              transformOrigin: "left center",
              display: "inline-block",
            }}
          >
            spikes.sh
          </span>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
