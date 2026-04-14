import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from "remotion";
import { theme } from "../theme";

const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

const ADD_CMD = 'claude mcp add spikes "npx -y spikes-mcp"';
const USER_Q1 = "what spikes are still open?";
const USER_Q2 = "yes, fix everything";

const CLAUDE_REPLY_1 = "3 open · hero CTA worst (2 'no'). Fix all?";
const CLAUDE_REPLY_2 = "Done. 3 resolved.";

const typeString = (text: string, startFrame: number, frame: number, fpc = 2) =>
  text.slice(
    0,
    Math.max(0, Math.min(text.length, Math.floor((frame - startFrame) / fpc))),
  );

export const MCP: React.FC = () => {
  const frame = useCurrentFrame();
  const caret = Math.floor(frame / 12) % 2 === 0;

  // Stage 1 (terminal): 0-150
  const stage1Opacity = interpolate(frame, [0, 12, 145, 165], [0, 1, 1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Stage 2 (chat): 150-600
  const stage2Opacity = interpolate(frame, [155, 185], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const addCmd = typeString(ADD_CMD, 12, frame, 2);
  const addCmdDone = frame >= 12 + ADD_CMD.length * 2;
  const addedOpacity = interpolate(frame, [95, 120], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Chat events — slowed for breathing room
  const q1 = typeString(USER_Q1, 210, frame, 2);
  const q1Done = frame >= 210 + USER_Q1.length * 2;
  const reply1Opacity = interpolate(frame, [330, 365], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const q2 = typeString(USER_Q2, 410, frame, 2);
  const q2Done = frame >= 410 + USER_Q2.length * 2;
  const reply2Opacity = interpolate(frame, [490, 525], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // MCP chip: pulses gently throughout chat
  const chipOpacity = interpolate(frame, [185, 215], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const dotPulse =
    0.55 + 0.45 * Math.sin(((frame - 185) / 30) * Math.PI * 1.2);

  const captionOpacity = interpolate(
    frame,
    [200, 240, 560, 600],
    [0, 1, 1, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  return (
    <AbsoluteFill
      style={{ backgroundColor: theme.bg, fontFamily: theme.font }}
    >
      {/* Stage 1 */}
      <AbsoluteFill
        style={{
          opacity: stage1Opacity,
          padding: 100,
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <div
          style={{
            width: "100%",
            background: theme.surface,
            border: `1px solid ${theme.border}`,
            borderRadius: 16,
            overflow: "hidden",
            boxShadow: "0 40px 120px rgba(0,0,0,0.6)",
          }}
        >
          <div
            style={{
              padding: "16px 22px",
              borderBottom: `1px solid ${theme.border}`,
              color: theme.muted,
              fontSize: 18,
              display: "flex",
              gap: 10,
              alignItems: "center",
            }}
          >
            <span style={{ width: 12, height: 12, borderRadius: 6, background: "#fb7185" }} />
            <span style={{ width: 12, height: 12, borderRadius: 6, background: "#fbbf24" }} />
            <span style={{ width: 12, height: 12, borderRadius: 6, background: "#4ade80" }} />
            <span style={{ marginLeft: 14 }}>~/my-site</span>
          </div>
          <div
            style={{
              padding: 56,
              color: theme.text,
              fontSize: 32,
              lineHeight: 1.6,
            }}
          >
            <div>
              <span style={{ color: theme.green }}>$</span>{" "}
              <span>{addCmd}</span>
              {!addCmdDone && caret && (
                <span
                  style={{
                    display: "inline-block",
                    width: 12,
                    height: 32,
                    marginLeft: 2,
                    background: theme.text,
                    verticalAlign: "middle",
                  }}
                />
              )}
            </div>
            <div
              style={{
                marginTop: 24,
                opacity: addedOpacity,
                color: theme.green,
                display: "flex",
                gap: 16,
                alignItems: "center",
              }}
            >
              <span>✓ added</span>
              <span style={{ color: theme.muted, fontSize: 26 }}>
                spikes · 9 tools
              </span>
            </div>
          </div>
        </div>
      </AbsoluteFill>

      {/* Stage 2: Claude chat */}
      <AbsoluteFill
        style={{
          opacity: stage2Opacity,
          padding: 100,
          flexDirection: "column",
        }}
      >
        <div
          style={{
            flex: 1,
            background: theme.surface,
            border: `1px solid ${theme.border}`,
            borderRadius: 16,
            overflow: "hidden",
            display: "flex",
            flexDirection: "column",
            boxShadow: "0 40px 120px rgba(0,0,0,0.6)",
            position: "relative",
          }}
        >
          <div
            style={{
              padding: "16px 22px",
              borderBottom: `1px solid ${theme.border}`,
              color: theme.muted,
              fontSize: 18,
              display: "flex",
              gap: 10,
              alignItems: "center",
            }}
          >
            <span style={{ width: 10, height: 10, borderRadius: 5, background: theme.red }} />
            Claude Code
          </div>
          <div
            style={{
              flex: 1,
              padding: 56,
              fontSize: 30,
              lineHeight: 1.5,
              color: theme.text,
              display: "flex",
              flexDirection: "column",
              gap: 28,
              justifyContent: "center",
            }}
          >
            {frame >= 210 && (
              <div
                style={{
                  alignSelf: "flex-end",
                  maxWidth: "70%",
                  background: theme.bg,
                  border: `1px solid ${theme.border}`,
                  borderRadius: 14,
                  padding: "20px 26px",
                }}
              >
                {q1}
                {!q1Done && caret && (
                  <span
                    style={{
                      display: "inline-block",
                      width: 12,
                      height: 30,
                      marginLeft: 2,
                      background: theme.text,
                      verticalAlign: "middle",
                    }}
                  />
                )}
              </div>
            )}

            {frame >= 330 && (
              <div
                style={{
                  alignSelf: "flex-start",
                  maxWidth: "70%",
                  background: theme.bg,
                  border: `1px solid ${theme.border}`,
                  borderRadius: 14,
                  padding: "20px 26px",
                  opacity: reply1Opacity,
                }}
              >
                {CLAUDE_REPLY_1}
              </div>
            )}

            {frame >= 410 && (
              <div
                style={{
                  alignSelf: "flex-end",
                  maxWidth: "60%",
                  background: theme.bg,
                  border: `1px solid ${theme.border}`,
                  borderRadius: 14,
                  padding: "20px 26px",
                }}
              >
                {q2}
                {!q2Done && caret && (
                  <span
                    style={{
                      display: "inline-block",
                      width: 12,
                      height: 30,
                      marginLeft: 2,
                      background: theme.text,
                      verticalAlign: "middle",
                    }}
                  />
                )}
              </div>
            )}

            {frame >= 490 && (
              <div
                style={{
                  alignSelf: "flex-start",
                  maxWidth: "70%",
                  background: theme.bg,
                  border: `1px solid ${theme.border}`,
                  borderRadius: 14,
                  padding: "20px 26px",
                  opacity: reply2Opacity,
                  color: theme.green,
                  fontWeight: 700,
                }}
              >
                {CLAUDE_REPLY_2}
              </div>
            )}
          </div>

          {/* MCP chip — bottom right with pulsing dot */}
          <div
            style={{
              position: "absolute",
              bottom: 28,
              right: 28,
              opacity: chipOpacity,
              background: theme.bg,
              border: `1px solid ${theme.border}`,
              borderRadius: 999,
              padding: "10px 18px",
              display: "flex",
              alignItems: "center",
              gap: 10,
              fontSize: 18,
              color: theme.muted,
            }}
          >
            <span
              style={{
                width: 10,
                height: 10,
                borderRadius: 5,
                background: theme.red,
                boxShadow: `0 0 0 ${4 * dotPulse}px rgba(231,76,60,${0.25 * dotPulse})`,
              }}
            />
            via spikes-mcp · 9 tools
          </div>
        </div>
      </AbsoluteFill>

      {/* Caption */}
      <div
        style={{
          position: "absolute",
          left: "50%",
          bottom: 36,
          transform: "translateX(-50%)",
          opacity: captionOpacity,
          color: theme.muted,
          fontSize: 26,
          letterSpacing: 1,
        }}
      >
        Or skip the paste step entirely.
      </div>
    </AbsoluteFill>
  );
};
