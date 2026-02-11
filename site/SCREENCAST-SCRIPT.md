# Spikes Screencast Script

**Duration:** 35-40 seconds  
**Resolution:** 1920×1080 (16:9)  
**FPS:** 30fps (60fps for smoother mouse movements)  
**Format:** MP4 (H.264)

---

## Pre-Recording Setup

### Demo Files Needed
Create a simple mockup directory with one HTML file:

```bash
mkdir -p ~/demo-mockup
```

**~/demo-mockup/pricing.html:**
```html
<!DOCTYPE html>
<html>
<head>
  <title>Pricing Page</title>
  <style>
    body { font-family: system-ui; max-width: 800px; margin: 60px auto; padding: 20px; }
    h1 { color: #1a1a1a; }
    .plan { border: 2px solid #e0e0e0; border-radius: 12px; padding: 24px; margin: 20px 0; }
    .plan h2 { margin-top: 0; }
    .cta { background: #3b82f6; color: white; border: none; padding: 12px 24px; 
           border-radius: 8px; font-size: 16px; cursor: pointer; }
    .price { font-size: 32px; font-weight: bold; }
  </style>
</head>
<body>
  <h1>Pricing</h1>
  <div class="plan">
    <h2>Pro Plan</h2>
    <p class="price">$29/mo</p>
    <p>Everything you need to get started.</p>
    <button class="cta">Get Started</button>
  </div>
</body>
</html>
```

### Terminal Setup
- Font size: 16-18pt (readable on video)
- Theme: Dark background with light text
- Window size: ~800×400px alongside browser
- Pre-run: `cd ~/demo-mockup && spikes init`

### Browser Setup
- Window size: ~1100×800px
- Clear address bar history
- Close unnecessary tabs
- Ensure widget is injected: `spikes inject ~/demo-mockup`

### Recording Tools
- **macOS:** QuickTime (simple) or OBS (more control)
- **OBS Settings:** 1920×1080, 30fps, CRF 18-20 for quality

---

## Script

### 0:00-0:05 — The Problem
**Screen:** Browser showing `pricing.html` mockup  
**Action:** Mouse moves slowly across the page, highlighting there's no feedback mechanism  
**Optional VO:** *"You've built a mockup. How do you get feedback?"*

---

### 0:05-0:10 — Activate Spikes
**Screen:** Same browser view  
**Action:**
1. Move cursor to the 🗡️ button (bottom-right corner)
2. Pause for 0.5s (let viewers see it)
3. Click the button
4. Button starts pulsing — cursor changes to crosshair

**Note:** Show the crosshair cursor clearly; this visual signals "spike mode"

---

### 0:10-0:17 — Element Feedback
**Screen:** Browser in spike mode  
**Action:**
1. Hover over the "Get Started" button — red highlight outline appears
2. Click the button
3. Popover appears anchored to the element showing: `button.cta`
4. Click the 😐 Meh rating button
5. Type in comment field: `Needs better contrast`
6. Click Save — button flashes green "✓ Saved!"

**Note:** Keep typing brief; viewers should read the comment easily

---

### 0:17-0:22 — Page Feedback
**Screen:** Browser, widget closed  
**Action:**
1. Click the 🗡️ button again
2. Click the button itself (not an element) to open page modal
3. Click ❤️ Love rating
4. Type: `Clean layout`
5. Click Save

**Note:** This shows both element AND page feedback workflows

---

### 0:22-0:32 — CLI Query
**Screen:** Split view or transition to terminal (terminal on left, browser visible on right)  
**Action:**
1. Terminal shows prompt in `~/demo-mockup`
2. Type: `spikes list`
3. Press Enter
4. Output shows table with both spikes (element + page)
5. Brief pause to show the output
6. Type: `spikes list --json | jq '.[].selector'`
7. Press Enter
8. Output shows: `"button.cta"` and `null` (for page spike)

**Terminal Output Preview:**
```
┌───────────────────────────────────────────────────────────────┐
│ ID       │ Type    │ Page         │ Rating │ Reviewer        │
├───────────────────────────────────────────────────────────────┤
│ a1b2c3d4 │ element │ Pricing Page │ meh    │ Demo User       │
│ e5f6g7h8 │ page    │ Pricing Page │ love   │ Demo User       │
└───────────────────────────────────────────────────────────────┘
```

---

### 0:32-0:37 — Closing
**Screen:** Terminal or browser  
**Option A — Show URL:**
- Browser address bar shows: `spikes.sh`
- Quick glimpse of the landing page

**Option B — Show Install:**
- Terminal: `curl -fsSL spikes.sh/install.sh | sh`
- Shows download/install progress

**Option C — Simple Tag:**
- Show text overlay: `spikes.sh` with the 🗡️ icon
- Clean, memorable

---

## Alternative: Condensed 30-Second Version

If 40 seconds is too long, cut the page feedback section (0:17-0:22):

| Time | Content |
|------|---------|
| 0:00-0:03 | Show mockup |
| 0:03-0:08 | Click button, enter spike mode |
| 0:08-0:18 | Spike element, add comment, save |
| 0:18-0:27 | Terminal: `spikes list`, show output |
| 0:27-0:30 | Closing: spikes.sh URL |

---

## Recording Tips

1. **Practice the flow** 2-3 times before recording
2. **Move the mouse slowly** — quick movements look jarring on video
3. **Pause briefly** after each action so viewers can process
4. **Keep typing speed moderate** — not too fast, not too slow
5. **Clear the terminal** before starting (`clear` command)
6. **Pre-type commands** in a text file, paste during recording if needed
7. **Record in multiple takes** if needed — you can edit in post

## Post-Production

1. Trim any dead time at start/end
2. Add subtle fade-in/fade-out (0.5s each)
3. Consider adding a subtle background music track (optional)
4. Export at high quality for web hosting (H.264, CRF 20-23)
5. Create a poster frame from a good moment (for video thumbnail)

---

## Voiceover Script (Optional)

If adding narration:

> *"Got a mockup? Add one script tag."*  
> *[Click button]*  
> *"Click an element..."*  
> *[Spike the button, add comment]*  
> *"Leave feedback right where it matters."*  
> *[Switch to terminal]*  
> *"Query it from the CLI."*  
> *[Show spikes list]*  
> *"Spikes. Feedback for mockups."*  
> *[Show URL]*

Keep voiceover sparse — the visual demo speaks for itself.
