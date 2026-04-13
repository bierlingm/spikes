# video/

90-second Remotion demo for spikes.sh. See [STORYBOARD.md](./STORYBOARD.md) for the full beat-by-beat plan.

## Status

| Scene | State |
|---|---|
| 0. Title (0:00-0:05) | Stubbed in `src/scenes/Title.tsx` — animates the landing hero |
| 1. Broken loop (0:05-0:20) | Stubbed in `src/scenes/Problem.tsx` — mock, chat bubble, timer |
| 2. Widget capture (0:20-0:40) | Storyboarded only |
| 3. Feed to agent (0:40-0:60) | Storyboarded only |
| 4. MCP skip paste (0:60-0:80) | Storyboarded only |
| 5. Loop closed + CTA (0:80-0:90) | Storyboarded only |

## Run

```bash
cd video
npm install       # or bun / pnpm
npm run dev       # opens Remotion Studio to iterate on scenes visually
npm run render    # outputs out/spikes-demo.mp4
```

## Adding a new scene

1. Add `src/scenes/Whatever.tsx` exporting a `React.FC`.
2. Append it to `SCENES` in `src/Root.tsx` with its duration.
3. Open Remotion Studio (`npm run dev`), select the composition, scrub to your scene's start frame.

## Design constraints

- Scenes are **30 fps, 1920×1080**. Use `useCurrentFrame()` relative to the scene start (Remotion resets frame count per `Series.Sequence`).
- Use `interpolate(frame, [a, b], [from, to], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })` for any animated value. Never raw arithmetic with frame.
- Theme tokens live in `src/theme.ts`. Don't hardcode hex values in scenes.
- Fonts: Berkeley Mono preferred; falls back to system mono. No external font load — ship fonts locally if needed.

## Rendering for publication

- **Landing page embed:** 1280×720 MP4, 4 Mbps VBR, H.264.
- **YouTube:** 1920×1080, 8-10 Mbps, H.264, AAC audio.
- **Twitter/X:** same as YouTube but keep under 100 MB and 2:20.
- **Loop GIF for HN/docs:** First 10 seconds, palette-optimized via `gifski` post-render.

Commands once `out/spikes-demo.mp4` exists:

```bash
# 720p web version
ffmpeg -i out/spikes-demo.mp4 -vf "scale=1280:720" -c:v libx264 -b:v 4M -an out/spikes-demo-720.mp4

# loop gif
ffmpeg -i out/spikes-demo.mp4 -t 10 -vf "fps=15,scale=960:-1" -f image2pipe -vcodec ppm - | gifski -o out/spikes-demo.gif --fps 15 -
```

## What's missing before ship

- Scenes 2–5 built out (biggest remaining work — probably 4-6 hours of iteration in Remotion Studio).
- Audio bed. Either instrumental stock music (Epidemic Sound, Artlist) or original. Target 80 BPM, minimal synth.
- Caption `.vtt` file for accessibility.
- Real CTA page mockup instead of the placeholder inline in `Problem.tsx` — ideally screenshot of actual `site/index.html` styled page.
