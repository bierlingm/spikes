# Spikes Brand Guide

## Voice

Spikes speaks like a well-made tool: direct, precise, and quietly confident. The voice is that of a craftsperson who respects your time—no sales pitch, no filler, just what you need to know.

**Sharp, not cute.** Where others soften with emoji and exclamation marks, Spikes cuts straight. "Feedback that cuts through" not "Get amazing feedback!" The sharpness is in the precision, not aggression.

**Generous, not transactional.** The pricing philosophy (Hierophant reversed + Queen of Pentacles) infuses everything: give freely, ask warmly, trust people. "Use it free forever, pay when it earns its place" not "Start your free trial."

**Technical, not corporate.** Speak to people who ship HTML and pipe JSON. CLI examples over marketing promises. "Every command outputs JSON" is more compelling than "powerful integrations."

## What We Say

- "Pinpoint any element" not "intuitive interface"
- "Zero friction" not "seamless experience"
- "Your data, your control" not "enterprise-grade security"
- "8KB gzipped" not "lightning fast"
- "Spike us back" not "Support our work"

## What We Don't Say

Never:
- "Game-changer" / "Revolutionary" / "Cutting-edge"
- "Seamless" / "Effortless" / "Magical"
- "Unlock" / "Unleash" / "Supercharge"
- "Get started for free!" (just "Free. Forever.")
- Exclamation marks in headlines

---

## Emoji & Icon Policy

### The Principle
Emoji serve function or reinforce the brand vocabulary. They are not decoration.

### Allowed Emoji

**The Sword (🗡️):** Brand mark. Used in favicon, logo position. Reinforces "sharp" metaphor without forcing it.

**Rating Emoji (❤️👍😐👎):** Functional—they communicate rating levels in the widget. Consider replacing with typographic alternatives in copy.

**Supporter Tiers (💚💙):** These work because they tie directly to the "spike us back" concept—the emoji ARE the rating. The heart is a spike. Keep these.

### Replace or Remove

| Current | Replace With |
|---------|--------------|
| 💬 (widget button) | 🗡️ or typographic mark |
| 🎯⚡🤖🏠📊🪶 (feature icons) | Simple SVG shapes or typographic glyphs |
| 🎨🤝🧪📋 (use case icons) | Remove entirely or use minimal shapes |

### Badge Icons
The current badges are clean. Keep them typographic.

---

## Color Palette

### Primary
- **Spike Red:** #e74c3c (active, buttons, emphasis)
- **Spike Red Dark:** #c0392b (hover states)
- **Spike Red Glow:** rgba(231, 76, 60, 0.4)

### Background (Dark Mode)
- **Deep Black:** #09090b (base)
- **Subtle Black:** #0f0f11 (cards)
- **Card Black:** #141417 (elevated surfaces)

### Text
- **Primary:** #fafafa (headlines, body)
- **Muted:** #71717a (secondary text)
- **Dim:** #52525b (tertiary, placeholders)

### Semantic
- **Green (Love/Success):** #22c55e
- **Blue (Agency):** #3b82f6
- **Yellow (Warning):** #eab308

### Guidance
The palette is tight. Resist adding colors. If something needs emphasis, use red or adjust typography weight—not a new color.

---

## Typography

### Current (Keep)
System font stack: `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`

This is correct for a developer tool. No custom fonts loading, instant render, familiar feel.

### Code/Mono
`'SF Mono', Monaco, 'Cascadia Code', monospace`

Used for CLI examples, selectors, code blocks.

### Hierarchy
- **Hero:** clamp(40px, 8vw, 72px), weight 700, tracking -0.02em
- **Section title:** clamp(32px, 5vw, 48px), weight 700
- **Body:** 15-17px, line-height 1.6
- **Code:** 13-14px

### Guidance
The type system is utilitarian. This is right. Don't add a display font—it would make Spikes feel like a marketing site instead of a tool.

---

## The Sharp Aesthetic

### Tool vs. Blade
The question was raised: should Spikes feel more "tool" (utilitarian) or "blade" (elegant danger)?

**Answer: Tool that happens to be a blade.**

- Clean interfaces, not ornate ones
- Functional animations (feedback confirmation), not decorative ones
- Sparse use of the red accent—when it appears, it cuts
- No gradients except the subtle red-to-orange on the logo position
- Generous whitespace—the sharpness is in what's absent

### Visual Signatures
1. **Red accent line** — The 4px red bar at the top of pricing cards. Consider making this a consistent element.
2. **Crosshair cursor** — In spike mode. The tool becomes a weapon becomes a tool.
3. **Pulse animation** — The button pulses when armed. Restrained but alive.

---

## Applying the Brand

### Headlines
Before: "Sharp tools for sharp feedback"
After: "Precise feedback. Zero friction."

Before: "Three steps to better feedback"  
After: "How it works"

### Feature Descriptions
Before: "Element-level precision 🎯"
After: "Element-level precision"

Before: "Click any element to spike it. Captures CSS selector, bounding box, and text content. No more 'the thing on the left.'"
After: (keep—this is already good)

### Buttons
Before: "Get Started →"
After: "Get Started" (no arrow emoji, use CSS arrow if needed)

### Trust Section
Before: "Built for the way you work"
After: Remove this line. Let the badges speak.

### Microcopy
Consistent patterns:
- Buttons: Sentence case, no punctuation ("Save", "Cancel", "Get Started")
- Labels: Sentence case ("Export as JSON")
- CLI examples: Actual commands, not descriptions

---

## Logo Mark

### Current: 🗡️ Emoji
The sword emoji works in context but has limitations:
- Renders differently across platforms
- Can't be customized (color, weight)
- Feels informal in some contexts

### Recommendation
**Phase 1 (Now):** Keep 🗡️ for favicon and casual use. It's recognizable and fits the indie vibe.

**Phase 2 (If Spikes grows):** Commission a simple SVG mark—a minimal sword or spike shape that can be:
- Single color (works in monotone contexts)
- Scaled without quality loss
- Used in badges, print, etc.

The mark should be geometric, not illustrative. Think: what if Stripe made a sword icon?

---

## Checklist

### Voice
- [ ] No generic SaaS phrases
- [ ] No exclamation marks in headlines
- [ ] Technical specifics over marketing promises
- [ ] Warm but not cute

### Emoji
- [ ] 🗡️ in brand position only
- [ ] Rating emoji (❤️👍😐👎) acceptable in widget
- [ ] Supporter tier emoji (💚💙😐) acceptable
- [ ] No decorative emoji in features/use cases
- [ ] Widget button: 🗡️ not 💬

### Visual
- [ ] Red accent used sparingly
- [ ] Dark backgrounds, high contrast text
- [ ] System fonts throughout
- [ ] No custom display fonts
- [ ] Generous whitespace

### Copy
- [ ] Every word earns its place
- [ ] CLI examples are real commands
- [ ] "Spike us back" explained in context
- [ ] Free tier emphasized without asterisks

---

## Visual Assets Needed

### Immediate (v1)
None required. The 🗡️ emoji works as logo for now.

### If Spikes Grows (v2)

| Asset | Purpose | Spec |
|-------|---------|------|
| **Logo mark (SVG)** | Favicon, badges, print | Minimal sword/spike, single color, 24x24 base |
| **Logo wordmark (SVG)** | Header, formal contexts | "Spikes" in system font + mark |
| **OG image** | Social sharing | 1200x630, dark bg, tagline + mark |
| **Feature icons (SVG)** | Replace current text glyphs if needed | 24x24, single stroke, matches mark style |

### Not Needed
- Custom fonts
- Illustration style
- Complex icon set
- Animation assets
- Print collateral

The brand is the voice and the red. Everything else stays minimal.
