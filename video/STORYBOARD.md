# Spikes demo video — storyboard

**Length:** 90 seconds
**Format:** 1920×1080, 30fps, MP4 (H.264)
**Engine:** Remotion (React-based, composition in `src/scenes/`)
**Voice:** No voiceover. On-screen captions only. Audio is a minimal ticking/synth bed (royalty-free, see `audio/` — TBD).
**Palette:**
  - Background: `#09090b` (near-black)
  - Surface: `#141417`
  - Text: `#fafafa`
  - Muted: `#a1a1aa`
  - Brand red: `#e74c3c` (accent only, never wash)
  - Green: `#22c55e` (resolved state)
**Font:** Berkeley Mono (already loaded by site/). Fallback `ui-monospace`.

The video follows the landing page thesis: "Building is easy. The feedback loop isn't." Each scene is a beat of that argument.

---

## Scene 0 — Title card (0:00 – 0:05)

**Background:** Solid `#09090b`.

**Text animation:**
- Line 1 fades in: `Building is easy now.`
- Line 2 wipes in 0.3s later (red gradient), replacing it: `The feedback loop isn't.`
- Sword mark (`/`) spikes in to the left of line 2 on the last 10 frames.

**Feel:** Quiet. Slightly too slow on purpose — sets up that this isn't a hype video.

---

## Scene 1 — The broken loop (0:05 – 0:20)

**Visual:** Mockup of a pricing page (use `~/demo-mockup/pricing.html` content from `site/SCREENCAST-SCRIPT.md` as the source — same design language).

**Beats:**
1. `0:05` — Mockup fades in, centered, mac-window chrome.
2. `0:07` — A chat bubble slides in from the right: *"the button thing is off"* (iMessage-grey, no sender, universal).
3. `0:09` — A screenshot thumbnail slides in beneath the bubble with a crude red arrow drawn on the CTA button. Caption: *"add some details?"* from the user.
4. `0:12` — Timer counter appears in the top-right: `00:00:00` starts counting up. Font-feature `tabular-nums`.
5. `0:17` — Counter reaches `00:40:00` as it fades. Text fades in: **"40 minutes. One padding value."**

**Emotion target:** Wince. Viewer recognizes themselves.

---

## Scene 2 — Widget capture (0:20 – 0:40)

**Visual:** Same pricing page, but now full-screen. Sword pin icon in bottom-right corner pulses faintly.

**Beats:**
1. `0:20` — Viewer's cursor enters frame, moves to sword pin.
2. `0:22` — Click. Pin turns red. Cursor changes to crosshair.
3. `0:24` — Cursor moves to the CTA button. Button gets a red outline highlight.
4. `0:26` — Click. Rating dialog slides up from the bottom: four reactions (`♥ love` / `↑ like` / `· meh` / `↓ no`).
5. `0:29` — Cursor picks `↓ no`. Dialog expands with a comment field.
6. `0:31` — Text types in (character-by-character animation): *"too dim on mobile"*.
7. `0:34` — Enter. Dialog slides down. Small toast: *"spiked."*
8. `0:37` — Camera zooms *into* the pin, which morphs into a JSON object. The object animates line-by-line:

```json
{
  "selector": ".hero > .cta-group > .btn-primary",
  "element_text": "Get Started",
  "rating": "no",
  "comment": "too dim on mobile",
  "viewport": { "width": 375, "height": 812 },
  "bounding_box": { "x": 412, "y": 688, "width": 156, "height": 44 }
}
```

**Caption through this whole scene (small, bottom-left):** *"Click. Rate. Done."*

---

## Scene 3 — Feed to the agent (0:40 – 0:60)

**Visual:** Split screen. Left half: terminal. Right half: Claude Code chat UI (use Claude's actual visual language, dark theme).

**Beats:**
1. `0:40` — Terminal appears on left. Prompt shows `~/my-site $`.
2. `0:42` — Types: `spikes list --json --unresolved`.
3. `0:44` — JSON spike output scrolls in (same object as scene 2, plus a second spike for variety).
4. `0:47` — User runs `| pbcopy`. Small "copied" indicator.
5. `0:49` — Right side: Claude chat, already open. User types: *"Fix these and mark them resolved."* Paste animation.
6. `0:53` — Claude response streams: *"Reading 2 spikes. Starting with the hero CTA (2 'no' ratings)."* Then: *"Editing `index.html`…"* then a diff snippet renders.
7. `0:57` — Small green checkmark next to each spike in the left panel: `✓ resolved`.

**Caption:** *"Paste. Fix. Mark resolved."*

---

## Scene 4 — MCP: skip the paste (0:60 – 0:80)

**Visual:** Terminal takes the full frame.

**Beats:**
1. `1:00` — Command: `claude mcp add spikes "npx -y spikes-mcp"`. Green `✓ added`.
2. `1:03` — Cut to Claude chat (full frame this time, no split).
3. `1:05` — User types: *"what spikes are still open"*.
4. `1:07` — Claude: *"3 open on index.html, 1 on pricing. Hero CTA has 2 'no' ratings — start there?"*
5. `1:12` — User: *"yes, fix everything"*.
6. `1:14` — Claude's output streams: *"Editing 3 files. Resolved 3 spikes."*
7. `1:18` — Tiny callout top-right: `9 MCP tools` (can list a few: `list_spikes`, `resolve_spike`, `get_hotspots`).

**Caption:** *"Or skip the paste step entirely."*

---

## Scene 5 — Loop closed + CTA (0:80 – 0:90)

**Visual:** Return to the pricing page mockup, now with the CTA button visibly fixed (brighter, correctly sized on mobile viewport indicator).

**Beats:**
1. `1:20` — Page reloads (subtle crossfade). The old + new button briefly overlay to show the diff.
2. `1:22` — Four spike pins, all green (resolved), float up and vanish.
3. `1:24` — Centered text: `Build. Spike. Fix. Repeat.`
4. `1:27` — Below it: `curl spikes.sh/install | sh`
5. `1:29` — Below that: `spikes.sh` in the brand red.
6. `1:30` — Cut to black.

**Emotion target:** Satisfaction. The loop closed on-screen.

---

## Cuts and tempo notes

- Every scene transition is a **hard cut**, no fades between scenes (fades only *inside* a scene). Keeps the pace.
- Music: 80 BPM minimal synth bed. Volume dips 20% during typed-text beats so the keyboard sound breathes.
- Every text overlay respects the brand guide: no emoji except the sword and the four rating glyphs.

## Accessibility

- All captions are full-contrast white on near-black.
- No critical information is color-only — rating reactions have text labels *and* glyphs.
- Final video ships with a `.vtt` captions file generated from the caption text in each scene.

## Render plan

- Scene 0: fully scripted in `src/scenes/Title.tsx` (stub committed).
- Scene 1: fully scripted in `src/scenes/Problem.tsx` (stub committed).
- Scenes 2–5: storyboarded here, implemented in follow-up sessions. Each is its own file under `src/scenes/`.
- Compose with `<Series>` in `src/Root.tsx`.

Render final:

```bash
cd video
bun install
bun run render  # outputs video/out/spikes-demo.mp4
```
