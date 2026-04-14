import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from "remotion";
import { theme } from "../theme";
import { MobilePhone } from "../components/MobilePhone";

const FPS = 30;
const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

const formatTimer = (seconds: number): string => {
  const m = Math.floor((seconds % 3600) / 60).toString().padStart(2, "0");
  const s = Math.floor(seconds % 60).toString().padStart(2, "0");
  return `${m}:${s}`;
};

type Msg = {
  who: "you" | "claude";
  text: string;
  appearAt: number;
  jiggle?: boolean; // whether it triggers a phone jiggle
  origin?: { name: string; role: string }; // shows quoted source
};

const MESSAGES: Msg[] = [
  {
    who: "you",
    text: "btn looks broken on mobile",
    appearAt: 30,
    origin: { name: "Sarah", role: "designer" },
  },
  { who: "claude", text: "Made it 20% bigger.", appearAt: 90, jiggle: true },
  { who: "you", text: "no, the spacing", appearAt: 150 },
  { who: "claude", text: "Padded the container.", appearAt: 210, jiggle: true },
  { who: "you", text: "still cut off", appearAt: 270 },
  { who: "claude", text: "Reduced font-size.", appearAt: 330, jiggle: true },
  { who: "you", text: "ugh nvm", appearAt: 390 },
];

export const Problem: React.FC = () => {
  const frame = useCurrentFrame();

  const fadeIn = interpolate(frame, [0, 25], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Timer ticks 0..2400 (40min), ease-out so it visibly drags toward the end.
  const timerStart = 5;
  const timerEnd = 12 * FPS;
  const timerProgress = interpolate(frame, [timerStart, timerEnd], [0, 2400], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const timerColor =
    timerProgress > 2200 ? theme.red : timerProgress > 1500 ? theme.text : theme.muted;
  const timerOpacity = interpolate(
    frame,
    [0, 25, 12 * FPS, 13 * FPS],
    [0, 1, 1, 0.2],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  // Punchline at 12.5s — gives 2.5s for it to land before scene cut at 15s.
  const punchOpacity = interpolate(
    frame,
    [12.5 * FPS, 13 * FPS],
    [0, 1],
    { easing: EASE_OUT, extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const punchRise = interpolate(
    frame,
    [12.5 * FPS, 13 * FPS + 5],
    [12, 0],
    { easing: EASE_OUT, extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  // Phone dims when punchline arrives — focus shifts to text.
  const phoneDim = interpolate(frame, [12 * FPS, 12.5 * FPS], [1, 0.35], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Find latest jiggle-triggering message to drive a brief phone shake.
  const lastJiggle = MESSAGES.filter((m) => m.jiggle && frame >= m.appearAt)
    .map((m) => frame - m.appearAt)
    .reduce<number | null>(
      (acc, age) => (acc === null || age < acc ? age : acc),
      null,
    );
  const jiggleFrame = lastJiggle !== null && lastJiggle < 12 ? lastJiggle : 0;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: theme.bg,
        fontFamily: theme.font,
      }}
    >
      {/* Timer — top center */}
      <div
        style={{
          position: "absolute",
          top: 60,
          left: "50%",
          transform: "translateX(-50%)",
          opacity: timerOpacity,
          color: timerColor,
          fontSize: 96,
          fontWeight: 700,
          fontFeatureSettings: '"tnum"',
          letterSpacing: 4,
          transition: "color 300ms",
        }}
      >
        {formatTimer(timerProgress)}
      </div>

      {/* Phone — left */}
      <div
        style={{
          position: "absolute",
          left: 140,
          top: "50%",
          transform: "translateY(-50%)",
          opacity: fadeIn * phoneDim,
        }}
      >
        <MobilePhone fixedProgress={0} jiggleFrame={jiggleFrame} width={340} />
      </div>

      {/* Chat panel — right */}
      <div
        style={{
          position: "absolute",
          right: 140,
          top: 220,
          width: 760,
          maxHeight: 740,
          opacity: fadeIn,
          background: theme.surface,
          border: `1px solid ${theme.border}`,
          borderRadius: 16,
          overflow: "hidden",
          boxShadow: "0 30px 90px rgba(0,0,0,0.5)",
          display: "flex",
          flexDirection: "column",
        }}
      >
        <div
          style={{
            padding: "14px 20px",
            borderBottom: `1px solid ${theme.border}`,
            color: theme.muted,
            fontSize: 16,
            display: "flex",
            alignItems: "center",
            gap: 10,
          }}
        >
          <span style={{ width: 8, height: 8, borderRadius: 4, background: theme.red }} />
          Claude Code · my-site
        </div>
        <div
          style={{
            flex: 1,
            padding: "20px 22px",
            display: "flex",
            flexDirection: "column",
            gap: 14,
            overflow: "hidden",
          }}
        >
          {MESSAGES.map((m, i) => {
            const op = interpolate(frame, [m.appearAt, m.appearAt + 12], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            });
            const rise = interpolate(frame, [m.appearAt, m.appearAt + 14], [8, 0], {
              easing: EASE_OUT,
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            });
            const isYou = m.who === "you";
            return (
              <div
                key={i}
                style={{
                  alignSelf: isYou ? "flex-end" : "flex-start",
                  maxWidth: "85%",
                  opacity: op,
                  transform: `translateY(${rise}px)`,
                }}
              >
                {m.origin && (
                  <div
                    style={{
                      fontSize: 13,
                      color: theme.muted,
                      marginBottom: 6,
                      letterSpacing: 0.5,
                    }}
                  >
                    via Slack · {m.origin.name} ({m.origin.role})
                  </div>
                )}
                <div
                  style={{
                    background: theme.bg,
                    border: `1px solid ${m.origin ? theme.red : theme.border}`,
                    borderLeft: m.origin ? `3px solid ${theme.red}` : undefined,
                    borderRadius: 12,
                    padding: "12px 16px",
                    color: theme.text,
                    fontSize: 22,
                  }}
                >
                  <div
                    style={{
                      fontSize: 13,
                      color: isYou ? theme.muted : theme.red,
                      marginBottom: 4,
                      letterSpacing: 0.5,
                    }}
                  >
                    {isYou ? "you" : "Claude"}
                  </div>
                  {m.text}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Punchline */}
      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          bottom: 80,
          opacity: punchOpacity,
          transform: `translateY(${punchRise}px)`,
          color: theme.text,
          fontSize: 88,
          fontWeight: 700,
          letterSpacing: -2,
          textAlign: "center",
          lineHeight: 1.05,
        }}
      >
        40 minutes.
        <br />
        <span style={{ color: theme.muted, fontWeight: 500 }}>
          One padding value.
        </span>
      </div>
    </AbsoluteFill>
  );
};
