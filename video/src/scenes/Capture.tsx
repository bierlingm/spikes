import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame, Easing } from "remotion";
import { theme } from "../theme";
import { PricingMock } from "../components/PricingMock";
import { Cursor } from "../components/Cursor";

const EASE = Easing.bezier(0.4, 0, 0.2, 1);
const EASE_OUT = Easing.bezier(0.16, 1, 0.3, 1);

const COMMENT = "cuts off on mobile";

const JSON_LINES = [
  "{",
  '  "selector": ".btn-primary",',
  '  "rating": "no",',
  '  "comment": "cuts off on mobile",',
  '  "viewport": "375 × 812"',
  "}",
];

// Empirical screen-space coords (1920×1080, mockup width 1400 centered).
// Pin sits at mockup bottom-right; button at mid-card.
const PIN_TIP = { x: 1603, y: 723 };
const BUTTON_CENTER = { x: 434, y: 673 };
const NO_CHIP_CENTER = { x: 1229, y: 936 };
const CURSOR_START = { x: 220, y: 1010 };

export const Capture: React.FC = () => {
  const frame = useCurrentFrame();

  // Cursor follows arrow path while it's still an arrow (PIN segment),
  // then crosshair (centered) for BUTTON and NO_CHIP.
  // Arrow tip is offset (+1, +1) from SVG left/top; crosshair is centered on x/y.
  const cursorPathX = interpolate(
    frame,
    [0, 45, 105, 180, 260, 300, 450],
    [
      CURSOR_START.x,
      CURSOR_START.x + 240,
      PIN_TIP.x - 1,
      BUTTON_CENTER.x,
      BUTTON_CENTER.x,
      NO_CHIP_CENTER.x,
      NO_CHIP_CENTER.x,
    ],
    { easing: EASE, extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const cursorPathY = interpolate(
    frame,
    [0, 45, 105, 180, 260, 300, 450],
    [
      CURSOR_START.y,
      CURSOR_START.y - 80,
      PIN_TIP.y - 1,
      BUTTON_CENTER.y,
      BUTTON_CENTER.y,
      NO_CHIP_CENTER.y,
      NO_CHIP_CENTER.y,
    ],
    { easing: EASE, extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const crosshair = frame >= 105;
  const cursorOpacity = interpolate(frame, [440, 470], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Click "press" punch — brief scale-down at click moments
  const pressFrame = (clickFrame: number) => {
    const dt = frame - clickFrame;
    if (dt < 0 || dt > 12) return 1;
    return interpolate(dt, [0, 4, 12], [1, 0.82, 1], { easing: EASE_OUT });
  };
  const cursorScale = Math.min(pressFrame(105), pressFrame(290));

  // Pin pre-click pulse
  const pinPulse =
    frame < 105 ? 0.5 + 0.5 * Math.sin((frame / 30) * Math.PI * 1.6) : 0;

  // Pin red after click
  const pinRed = interpolate(frame, [100, 120], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const pinClickPunch = interpolate(frame, [105, 115, 125], [1, 1.18, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // CTA highlight
  const ctaHighlight = frame >= 175 && frame <= 470;

  // Dialog
  const dialogY = interpolate(frame, [200, 240], [200, 0], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const dialogOpacity = interpolate(
    frame,
    [200, 235, 430, 460],
    [0, 1, 1, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const selectedIdx = frame >= 290 ? 3 : -1;
  const selectFlash = interpolate(frame, [290, 310, 340], [0, 1, 0.6], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  // Comment field expand — smooth interpolation, no snap
  const expand = interpolate(frame, [310, 345], [0, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const typedChars = Math.max(
    0,
    Math.min(COMMENT.length, Math.floor((frame - 350) / 4)),
  );
  const typedText = COMMENT.slice(0, typedChars);
  const showCaret =
    frame >= 345 && frame < 440 && Math.floor(frame / 12) % 2 === 0;

  // JSON layer
  const jsonStart = 470;
  const mockoutOpacity = interpolate(frame, [jsonStart, jsonStart + 25], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const jsonContainerOpacity = interpolate(
    frame,
    [jsonStart + 8, jsonStart + 28],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const jsonScale = interpolate(frame, [jsonStart, jsonStart + 35], [0.92, 1], {
    easing: EASE_OUT,
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const jsonReadyTagOpacity = interpolate(
    frame,
    [jsonStart + 50, jsonStart + 75],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const captionOpacity = interpolate(
    frame,
    [40, 80, 460, 490],
    [0, 1, 1, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );

  const RATINGS = [
    { label: "love", glyph: "♥" },
    { label: "like", glyph: "↑" },
    { label: "meh", glyph: "·" },
    { label: "no", glyph: "↓" },
  ];

  return (
    <AbsoluteFill
      style={{
        backgroundColor: theme.bg,
        fontFamily: theme.font,
        overflow: "hidden",
      }}
    >
      {/* Stage layer (mockup + dialog) */}
      <AbsoluteFill style={{ opacity: mockoutOpacity }}>
        <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
          <div style={{ position: "relative" }}>
            <PricingMock width={1400} highlightCta={ctaHighlight} />
            <div
              style={{
                position: "absolute",
                right: 28,
                bottom: 28,
                transform: `scale(${pinClickPunch})`,
              }}
            >
              {pinPulse > 0 && (
                <div
                  style={{
                    position: "absolute",
                    inset: 0,
                    borderRadius: "50%",
                    boxShadow: `0 0 0 ${10 + pinPulse * 14}px rgba(231,76,60,${0.18 * pinPulse})`,
                  }}
                />
              )}
              <div
                style={{
                  width: 56,
                  height: 56,
                  borderRadius: "50%",
                  background: `rgb(${Math.round(9 + 222 * pinRed)}, ${Math.round(9 + 67 * pinRed)}, ${Math.round(11 + 49 * pinRed)})`,
                  color: "#fafafa",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: 30,
                  fontWeight: 800,
                  fontFamily: theme.font,
                  boxShadow: "0 12px 30px rgba(0,0,0,0.5)",
                }}
              >
                /
              </div>
            </div>
          </div>
        </AbsoluteFill>

        {/* Rating dialog */}
        <div
          style={{
            position: "absolute",
            left: "50%",
            bottom: 90,
            transform: `translate(-50%, ${dialogY}px)`,
            opacity: dialogOpacity,
            width: 760,
            background: theme.surface,
            border: `1px solid ${theme.border}`,
            borderRadius: 18,
            padding: 28,
            boxShadow: "0 40px 120px rgba(0,0,0,0.6)",
            zIndex: 1,
          }}
        >
          <div
            style={{
              color: theme.muted,
              fontSize: 13,
              letterSpacing: 1.5,
              textTransform: "uppercase",
              marginBottom: 18,
            }}
          >
            .btn-primary · “Get Started”
          </div>
          <div style={{ display: "flex", gap: 14 }}>
            {RATINGS.map((r, i) => {
              const selected = selectedIdx === i;
              return (
                <div
                  key={r.label}
                  style={{
                    flex: 1,
                    padding: "20px 12px",
                    borderRadius: 12,
                    border: `1px solid ${selected ? theme.red : theme.border}`,
                    background: selected
                      ? `rgba(231,76,60,${0.08 + 0.12 * selectFlash})`
                      : "transparent",
                    color: selected ? theme.red : theme.muted,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: 10,
                    fontSize: 22,
                    fontWeight: 600,
                    transition: "color 200ms",
                  }}
                >
                  <span>{r.glyph}</span>
                  <span>{r.label}</span>
                </div>
              );
            })}
          </div>
          <div
            style={{
              marginTop: 18 * expand,
              overflow: "hidden",
              maxHeight: 90 * expand,
              opacity: expand,
            }}
          >
            <div
              style={{
                background: theme.bg,
                border: `1px solid ${theme.border}`,
                borderRadius: 10,
                padding: "16px 18px",
                fontSize: 22,
                color: theme.text,
                display: "flex",
                alignItems: "center",
              }}
            >
              <span>{typedText}</span>
              {showCaret && (
                <span
                  style={{
                    display: "inline-block",
                    width: 2,
                    height: 24,
                    marginLeft: 3,
                    background: theme.red,
                  }}
                />
              )}
            </div>
          </div>
        </div>

        {/* Cursor — rendered LAST so it sits above dialog and pin */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            zIndex: 2,
            pointerEvents: "none",
            transform: `scale(${cursorScale})`,
            transformOrigin: `${cursorPathX}px ${cursorPathY}px`,
          }}
        >
          <Cursor
            x={cursorPathX}
            y={cursorPathY}
            variant={crosshair ? "crosshair" : "arrow"}
            opacity={cursorOpacity}
          />
        </div>

        {/* Caption */}
        <div
          style={{
            position: "absolute",
            left: 80,
            bottom: 50,
            opacity: captionOpacity,
            color: theme.muted,
            fontSize: 26,
            letterSpacing: 1,
          }}
        >
          Click. Rate. Done.
        </div>
      </AbsoluteFill>

      {/* JSON reveal layer */}
      <AbsoluteFill
        style={{
          alignItems: "center",
          justifyContent: "center",
          opacity: jsonContainerOpacity,
        }}
      >
        <div
          style={{
            transform: `scale(${jsonScale})`,
            background: theme.surface,
            border: `1px solid ${theme.border}`,
            borderRadius: 16,
            padding: "40px 56px",
            color: theme.text,
            fontSize: 36,
            lineHeight: 1.55,
            fontFamily: theme.font,
            minWidth: 1100,
            boxShadow: "0 40px 120px rgba(0,0,0,0.6)",
            position: "relative",
          }}
        >
          {JSON_LINES.map((line, i) => {
            const lineOpacity = interpolate(
              frame,
              [jsonStart + 18 + i * 4, jsonStart + 32 + i * 4],
              [0, 1],
              { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
            );
            return (
              <div
                key={i}
                style={{ whiteSpace: "pre", opacity: lineOpacity }}
              >
                {line}
              </div>
            );
          })}

          <div
            style={{
              position: "absolute",
              top: -16,
              right: 24,
              opacity: jsonReadyTagOpacity,
              background: theme.green,
              color: theme.bg,
              padding: "6px 14px",
              borderRadius: 999,
              fontSize: 18,
              fontWeight: 700,
              letterSpacing: 0.5,
            }}
          >
            agent-ready
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
