import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from "remotion";
import { theme } from "../theme";

const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

const COMMAND = "spikes list --json --unresolved";
const CLAUDE_PROMPT = "Fix these. Mark resolved.";

const JSON_OUTPUT = [
  "[",
  '  { "selector": ".btn-primary",       "rating": "no"   },',
  '  { "selector": ".pricing h2",        "rating": "meh"  }',
  "]",
];

const RESOLVED_FILES = [
  { file: "index.html", note: "hero CTA contrast" },
  { file: "pricing.html", note: "tier name overflow" },
];

const typeString = (text: string, startFrame: number, frame: number, fpc = 2) =>
  text.slice(
    0,
    Math.max(0, Math.min(text.length, Math.floor((frame - startFrame) / fpc))),
  );

export const FeedAgent: React.FC = () => {
  const frame = useCurrentFrame();
  const caret = Math.floor(frame / 12) % 2 === 0;

  // Terminal in 0-25
  const terminalOpacity = interpolate(frame, [0, 25], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  // Chat slides in 220-260, leftward push to balance space
  const chatOpacity = interpolate(frame, [220, 260], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const chatX = interpolate(frame, [220, 260], [60, 0], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const typed = typeString(COMMAND, 30, frame, 2);
  const cmdDone = frame >= 30 + COMMAND.length * 2;
  const jsonOpacity = interpolate(frame, [110, 150], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // pbcopy + copied badge
  const pbcopyVisible = frame >= 180;
  const copiedOpacity = interpolate(
    frame,
    [195, 215, 290, 310],
    [0, 1, 1, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  // Claude user types at 280
  const claudeTyped = typeString(CLAUDE_PROMPT, 280, frame, 2);
  const claudeTypedDone = frame >= 280 + CLAUDE_PROMPT.length * 2;
  const pasteOpacity = interpolate(frame, [380, 410], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Claude reply at 430
  const replyHeaderOpacity = interpolate(frame, [430, 460], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const file1Opacity = interpolate(frame, [470, 495], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const file2Opacity = interpolate(frame, [505, 530], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const summaryOpacity = interpolate(frame, [540, 565], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Terminal lines turn green ✓ as Claude resolves them
  const term1Resolved = interpolate(frame, [495, 520], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const term2Resolved = interpolate(frame, [530, 555], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const captionOpacity = interpolate(
    frame,
    [40, 80, 560, 600],
    [0, 1, 1, 0.7],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  return (
    <AbsoluteFill
      style={{
        backgroundColor: theme.bg,
        fontFamily: theme.font,
        padding: 70,
        gap: 36,
        flexDirection: "row",
        alignItems: "stretch",
      }}
    >
      {/* Terminal (left) */}
      <div
        style={{
          flex: 1,
          opacity: terminalOpacity,
          background: theme.surface,
          border: `1px solid ${theme.border}`,
          borderRadius: 16,
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          boxShadow: "0 30px 90px rgba(0,0,0,0.5)",
        }}
      >
        <div
          style={{
            padding: "16px 22px",
            borderBottom: `1px solid ${theme.border}`,
            display: "flex",
            alignItems: "center",
            gap: 10,
            color: theme.muted,
            fontSize: 16,
          }}
        >
          <span style={{ width: 12, height: 12, borderRadius: 6, background: "#fb7185" }} />
          <span style={{ width: 12, height: 12, borderRadius: 6, background: "#fbbf24" }} />
          <span style={{ width: 12, height: 12, borderRadius: 6, background: "#4ade80" }} />
          <span style={{ marginLeft: 14 }}>~/my-site</span>
        </div>
        <div
          style={{
            flex: 1,
            padding: 36,
            color: theme.text,
            fontSize: 24,
            lineHeight: 1.65,
            position: "relative",
          }}
        >
          <div>
            <span style={{ color: theme.green }}>$</span>{" "}
            <span>{typed}</span>
            {pbcopyVisible && (
              <span style={{ color: theme.muted }}> | pbcopy</span>
            )}
            {!cmdDone && caret && (
              <span
                style={{
                  display: "inline-block",
                  width: 10,
                  height: 24,
                  marginLeft: 2,
                  background: theme.text,
                  verticalAlign: "middle",
                }}
              />
            )}
          </div>

          {/* JSON output */}
          <div
            style={{
              marginTop: 22,
              opacity: jsonOpacity,
              fontSize: 20,
              lineHeight: 1.6,
              color: theme.muted,
              fontFamily: theme.font,
            }}
          >
            <div
              style={{
                color: term1Resolved > 0.1 ? theme.green : theme.muted,
                whiteSpace: "pre",
                transition: "color 200ms",
              }}
            >
              {term1Resolved > 0.5 ? "✓ " : "  "}
              {JSON_OUTPUT[1]}
            </div>
            <div
              style={{
                color: term2Resolved > 0.1 ? theme.green : theme.muted,
                whiteSpace: "pre",
                transition: "color 200ms",
              }}
            >
              {term2Resolved > 0.5 ? "✓ " : "  "}
              {JSON_OUTPUT[2]}
            </div>
          </div>

          {/* copied badge */}
          <div
            style={{
              position: "absolute",
              right: 28,
              top: 28,
              opacity: copiedOpacity,
              background: theme.bg,
              border: `1px solid ${theme.green}`,
              color: theme.green,
              padding: "8px 14px",
              borderRadius: 999,
              fontSize: 16,
              fontWeight: 600,
            }}
          >
            ✓ copied
          </div>
        </div>
      </div>

      {/* Claude chat (right) */}
      <div
        style={{
          flex: 1,
          opacity: chatOpacity,
          transform: `translateX(${chatX}px)`,
          background: theme.surface,
          border: `1px solid ${theme.border}`,
          borderRadius: 16,
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          boxShadow: "0 30px 90px rgba(0,0,0,0.5)",
        }}
      >
        <div
          style={{
            padding: "16px 22px",
            borderBottom: `1px solid ${theme.border}`,
            color: theme.muted,
            fontSize: 16,
            display: "flex",
            alignItems: "center",
            gap: 10,
          }}
        >
          <span style={{ width: 8, height: 8, borderRadius: 4, background: theme.red }} />
          Claude Code
        </div>
        <div
          style={{
            flex: 1,
            padding: 36,
            color: theme.text,
            fontSize: 24,
            lineHeight: 1.55,
            display: "flex",
            flexDirection: "column",
            gap: 22,
          }}
        >
          {frame >= 280 && (
            <div
              style={{
                alignSelf: "flex-end",
                maxWidth: "85%",
                background: theme.bg,
                border: `1px solid ${theme.border}`,
                borderRadius: 14,
                padding: "16px 20px",
              }}
            >
              <div>
                {claudeTyped}
                {!claudeTypedDone && caret && (
                  <span
                    style={{
                      display: "inline-block",
                      width: 10,
                      height: 24,
                      marginLeft: 2,
                      background: theme.text,
                      verticalAlign: "middle",
                    }}
                  />
                )}
              </div>
              <div
                style={{
                  marginTop: 12,
                  opacity: pasteOpacity,
                  fontSize: 15,
                  color: theme.muted,
                  borderLeft: `2px solid ${theme.red}`,
                  paddingLeft: 12,
                }}
              >
                pasted · 2 spikes
              </div>
            </div>
          )}

          {frame >= 430 && (
            <div
              style={{
                alignSelf: "flex-start",
                maxWidth: "92%",
                background: theme.bg,
                border: `1px solid ${theme.border}`,
                borderRadius: 14,
                padding: "20px 22px",
              }}
            >
              <div
                style={{
                  opacity: replyHeaderOpacity,
                  color: theme.muted,
                  fontSize: 16,
                  marginBottom: 14,
                }}
              >
                Editing 2 files…
              </div>
              {RESOLVED_FILES.map((f, i) => {
                const op = i === 0 ? file1Opacity : file2Opacity;
                return (
                  <div
                    key={f.file}
                    style={{
                      opacity: op,
                      display: "flex",
                      gap: 14,
                      alignItems: "baseline",
                      marginBottom: 8,
                    }}
                  >
                    <span style={{ color: theme.green, fontSize: 22 }}>✓</span>
                    <span style={{ color: theme.text }}>{f.file}</span>
                    <span style={{ color: theme.muted, fontSize: 18 }}>
                      · {f.note}
                    </span>
                  </div>
                );
              })}
              <div
                style={{
                  opacity: summaryOpacity,
                  marginTop: 14,
                  color: theme.green,
                  fontSize: 22,
                  fontWeight: 700,
                }}
              >
                2 resolved.
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Caption */}
      <div
        style={{
          position: "absolute",
          left: "50%",
          bottom: 24,
          transform: "translateX(-50%)",
          opacity: captionOpacity,
          color: theme.muted,
          fontSize: 26,
          letterSpacing: 1,
        }}
      >
        Paste. Fix. Mark resolved.
      </div>
    </AbsoluteFill>
  );
};
