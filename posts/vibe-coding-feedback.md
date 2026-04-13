# Your AI can't read your screenshot

**Primary title:** Your AI can't read your screenshot
**Alt titles (A/B if needed):**
- The missing primitive for vibe coding: structured UI feedback
- Stop describing UI bugs to Claude in English
- Feedback in, feedback out: the part of vibe coding nobody talks about

**Length:** ~1,400 words. Reading time 5 min.
**Audience:** AI-assisted builders — people shipping real things with Cursor, Claude Code, Windsurf, v0, Lovable.
**Suggested venues:** Personal blog / dev.to first (SEO and archive), HN Show / Ask (one shot), Twitter thread (snippets below).

---

You built a landing page with Claude in twenty minutes. You sent the link to a friend. Five minutes later you get back: *"the button thing is a bit off, and the middle section feels weird on mobile."*

You stare at it. Which button. Which middle. Feels weird how.

You ask your friend to send a screenshot. They send one, with a red arrow drawn on it. You paste the screenshot into Claude. Claude guesses which button. It guesses wrong. You copy the CSS class and paste that in. Claude asks for the viewport width. You don't know the viewport width your friend was on. You guess. You iterate. Forty minutes later the padding is finally right.

The **building** part took twenty minutes. The **feedback loop** took forty. That ratio is backwards, and it's going to get worse, not better, as AI keeps getting faster at the part that used to be hard.

The feedback loop is the unsolved problem of AI-assisted building.

## Why it's broken

Every AI coding tool today reads structured input brilliantly. Give Claude a DOM tree, a CSS selector, a bounding box, and a viewport, and it will change the right line of code on the first try. Give it *"the button thing is off"* and a screenshot, and you've just asked it to perform OCR, fuzzy matching, and ESP at once. It will get something wrong. Probably several things.

This isn't a model-capability problem. It's a **format** problem. Human-to-human UI feedback is lossy on purpose — language is imprecise because humans are forgiving. AI is not forgiving. AI is a machine that eats tokens and excretes diffs. It wants structure on the way in.

So the question is: how do your reviewers hand you structure without having to learn CSS?

## The primitive

The minimum viable feedback packet for an AI agent is something like this:

```json
{
  "selector": ".hero > .cta-group > .btn-primary",
  "element_text": "Get Started",
  "page": "index.html",
  "rating": "no",
  "comment": "too dim on mobile, also feels crowded next to the secondary CTA",
  "bounding_box": { "x": 412, "y": 688, "width": 156, "height": 44 },
  "viewport": { "width": 375, "height": 812 },
  "reviewer": { "name": "jamie" }
}
```

Your reviewer didn't type any of that. They clicked a button, picked a reaction, and typed *"too dim on mobile"* — the rest was captured automatically.

That packet is enough for Claude to:

1. Find the exact DOM node without guessing
2. Understand the visual layout via bounding box
3. Reason about *why* it's dim (the viewport was narrow)
4. Write a diff that fixes this element and only this element

No screenshot OCR. No CSS class hunting. No viewport ESP.

## What the loop looks like when it works

I've been shipping this pattern for a few months. Here's the current flow, end to end:

**Build with an AI.** Cursor, Claude Code, whatever. Normal prompt-driven development.

**Inject the widget.** One line at the bottom of your HTML:

```html
<script src="https://spikes.sh/widget.js" data-project="my-site"></script>
```

A small pin icon appears in the corner. Click it, click any element on the page, rate it, comment if you want. The widget records selector, bounding box, viewport, page, timestamp.

**Send the link to reviewers.** No login. No account. They click, rate, leave comments, close the tab.

**Pipe it to the AI.** In your terminal:

```bash
spikes list --json --unresolved | pbcopy
```

Paste into Claude with *"Here's the feedback from today's testers. Fix each one and mark it resolved."* Claude reads the JSON, makes a plan, opens the files, writes the diffs.

**Or skip the paste step.** Spikes ships an MCP server:

```bash
claude mcp add spikes "spikes mcp serve"
```

Now Claude can read, triage, and resolve feedback directly:

> me: what's unresolved
> claude: 3 spikes on index.html, 1 on pricing. the hero CTA one has 2 "no" ratings from different reviewers — probably worth starting there.
> me: fix it
> claude: *\[edits `.btn-primary` padding and contrast, marks spike resolved\]*

The loop went from *screenshot-and-guess* to *claude handles it.*

## What this isn't

**It isn't a form builder.** Typeform captures for humans to read later. Spikes captures for agents to act on now. The output format is the whole point.

**It isn't session replay.** FullStory, LogRocket, Hotjar — beautiful tools for a human analyst to watch a user struggle. Useless context to paste into Claude. The data is too big, too sequential, too video-shaped. Agents need anchored events, not streams.

**It isn't a linter.** Axe and Lighthouse catch machine-detectable problems. UI taste isn't machine-detectable. You need humans who pick a reaction and leave a half-sentence note.

**It isn't opinionated about where the feedback goes.** JSON in, do whatever you want with it. Pipe to Claude, dump into a spreadsheet, wire to a webhook, commit to git. The point is that the format survives the hand-off.

## The honest parts

Some things I haven't solved yet, in case you hit them:

- **Iframes and shadow DOM** are a nightmare for selector resolution. The widget handles shadow DOM reasonably and iframes badly. If you're embedding Stripe Elements or similar, expect the feedback to point at the iframe itself, not the input inside it.
- **SPAs with route changes** need a tiny hint — either the widget's `data-page` attribute or a manual `Spikes.setPage()` call on navigation. Otherwise every spike gets tagged with whatever URL was loaded first.
- **Mobile reviewers** are the whole ballgame and also where selector precision hurts most. A bounding box at `viewport: 375×812` is priceless. But your reviewer is on a phone, they're not going to type much — design your reactions to be tap-sized.

None of these are dealbreakers; they're the rough edges that tell you the tool is load-bearing rather than hypothetical.

## Try it

```bash
curl -fsSL https://spikes.sh/install | sh
spikes login
spikes inject ./my-project
# open the page, click the pin, you're spiking
```

The widget and CLI are free forever — MIT licensed, self-hostable. Paid tier is \$49 lifetime if you want unlimited shares, password-protected review links, webhooks, and API keys for agents. No subscriptions.

If you build with AI and you've felt the forty-minutes-fixing-a-button tax, I'd love to know if this closes the loop for you. You can spike the site itself: [spikes.sh](https://spikes.sh) has the widget loaded.

---

## Twitter thread variant (11 tweets)

**1/** Building a landing page with Claude takes 20 minutes. Getting the feedback right takes 40.

The AI part of AI-assisted building is solved. The feedback loop is not.

**2/** Your friend says "the button thing is off." You paste the screenshot into Claude. Claude guesses. Wrong button. You copy CSS. Claude asks the viewport. You don't know. You guess. Forty minutes later the padding is right.

**3/** This isn't a model problem. Claude reads structured input perfectly. It's a *format* problem.

Human-to-human feedback is lossy. AI doesn't forgive lossy. AI wants structure in.

**4/** The minimum packet Claude actually needs:
- CSS selector
- Bounding box
- Viewport
- Rating
- Optional short comment
- Page + timestamp

Reviewer types none of it. They click, pick a reaction, close the tab.

**5/** Drop-in widget, one line:
`<script src="https://spikes.sh/widget.js" data-project="my-site"></script>`

Pin appears. Click any element. Rate it. Comment if you feel like it.

**6/** Pipe it to your agent:
`spikes list --json --unresolved | pbcopy`

Paste into Claude: *"fix each of these and mark resolved."*

Claude opens the files. Writes the diffs. Done.

**7/** Or skip the paste step entirely. MCP server ships with the CLI:
`claude mcp add spikes "spikes mcp serve"`

Now Claude reads, triages, and resolves feedback without a middleman.

**8/** Not a form builder (that's for humans to read later)
Not session replay (too big to paste)
Not a linter (taste isn't machine-detectable)

It's an *anchored feedback event* — enough structure for an agent to act.

**9/** Rough edges I haven't solved:
- shadow DOM works, iframes don't
- SPAs need a tiny page hint on route change
- mobile = best viewport signal, worst typing UX

None are dealbreakers. They're the signs the tool is load-bearing.

**10/** Free forever for local workflows. \$49 lifetime if you want unlimited hosted shares + webhooks + agent API keys. No subscription.

**11/** Try it:
`curl -fsSL spikes.sh/install | sh`
`spikes login`
`spikes inject ./your-project`

spikes.sh has the widget loaded on its own page — you can spike the pitch itself.

---

## HN submission variants

**Show HN:** Spikes – Structured UI feedback your AI agent can actually act on
→ Link: https://spikes.sh
→ First comment: two-paragraph founder note (problem + why JSON-out-of-the-widget matters).

**Ask HN alt title (if "Show HN" feels too pitchy):** The missing primitive for AI-assisted building is structured feedback

---

## Drafting notes (keep/remove before publishing)

- Opening scene is specific on purpose. HN and dev.to both punish abstract openings. Keep the friend, the red arrow, the forty minutes.
- The JSON packet is the load-bearing evidence. Don't cut it.
- "Not a form builder / not session replay / not a linter" section exists to pre-empt the top HN comments. If you ship it to a venue where those comments aren't the risk (your own blog), you can compress it.
- Pricing mention is deliberately late and deliberately short. Post is not a pitch; the pitch is earned by then.
- Twitter thread is written to stand alone — every tweet should make sense if retweeted in isolation.
