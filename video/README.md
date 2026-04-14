# video/

90-second Remotion demo for spikes.sh. See [STORYBOARD.md](./STORYBOARD.md) for the full beat-by-beat plan.

## Status

| Scene | State |
|---|---|
| 0. Title (0:00-0:05) | `src/scenes/Title.tsx` |
| 1. Broken loop (0:05-0:20) | `src/scenes/Problem.tsx` |
| 2. Widget capture (0:20-0:40) | `src/scenes/Capture.tsx` |
| 3. Feed to agent (0:40-0:60) | `src/scenes/FeedAgent.tsx` |
| 4. MCP skip paste (0:60-0:80) | `src/scenes/MCP.tsx` |
| 5. Loop closed + CTA (0:80-0:90) | `src/scenes/Close.tsx` |

## Run

```bash
cd video
npm install
npm run dev          # Remotion Studio (visual iteration)
npm run typecheck    # tsc --noEmit
npm run render       # out/spikes-demo.mp4 (canonical 1920x1080, H.264)
npm run render:gif   # out/spikes-demo-loop.gif (short loop of Close scene)
npm run render:all   # all variants
```

## Outputs (`out/`)

- `spikes-demo.mp4` — canonical 1920×1080, 30fps, H.264. Embedded on `spikes.sh`.
- `spikes-demo-youtube.mp4` — CRF 18 re-encode for YouTube upload.
- `spikes-demo-twitter.mp4` — 1080×1080 square (letterboxed), for X/Twitter.
- `spikes-demo-loop.gif` — 10s Close scene, for README / HN previews.

All social variants stay under Twitter's 2:20 and size limits.

## Audio bed (optional)

Drop a royalty-free track at `public/audio/bed.mp3` and it wires in automatically at 35% volume. If the file is missing, the demo renders silent. Recommended sources: Pixabay Music, Uppbeat, Artlist. Target 80 BPM, minimal synth, ~90 seconds.

The renderer checks at runtime via `fetch(staticFile("audio/bed.mp3"), { method: "HEAD" })`, so there is nothing to configure — add or remove the file and re-render.

## Adding a scene

1. Add `src/scenes/Whatever.tsx` exporting a `React.FC`.
2. Append it to `SCENES` in `src/Root.tsx` with its duration in frames.
3. Open Remotion Studio (`npm run dev`), scrub to the scene's start frame.

## Design constraints

- 30 fps, 1920×1080. `useCurrentFrame()` is scene-local inside a `Series.Sequence`.
- Animate via `interpolate(frame, [a, b], [from, to], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })`. Never raw arithmetic with `frame`.
- Theme tokens live in `src/theme.ts`.
- Fonts: Berkeley Mono preferred, falls back to system mono.

## Publishing checklist

- [x] Canonical MP4 in `out/`
- [x] YouTube re-encode
- [x] Twitter square variant
- [x] Loop GIF
- [x] Landing embed at `site/spikes-demo.mp4`
- [ ] Audio bed (optional — drop at `public/audio/bed.mp3`)
- [ ] `.vtt` caption file
