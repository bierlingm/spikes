# Spikes License Model

## Philosophy

Spikes exists in the liminal space between designer and client, between "what do you think?" and "here's what I think." The whole product is about capturing *spikes* — moments of reaction, jabs of feeling.

The name says it all: sharp, quick, pointed.

The pricing model should match.

---

## The Cards We Drew

A tarot spread revealed the soul of this pricing:

- **The Hierophant (reversed)** — Breaking rules. Rebellion. Rejecting SaaS orthodoxy. *Keep rebelling.*
- **Queen of Pentacles** — Lavish. Generous. Warm-hearted. *Be generous anyway.*
- **Ace of Cups (reversed)** — Fear of one-sided relationships. Pouring out, nothing back. *Create reciprocity.*
- **Four of Swords** — The world wants peace. Rest from pricing complexity. *Be calm.*
- **Three of Cups** — Celebration. *Make payment feel joyful.*

---

## The Model

### Tier 0: Free

```
Everything. Forever. MIT licensed.

- Widget (<10KB, zero dependencies)
- CLI with JSON output for agents
- Local storage (works offline)
- HTML dashboard
- Deploy to YOUR Cloudflare/Vercel
- TUI dashboard

No accounts. No tracking. No feature gates.
No trial. No "free for personal use." No asterisks.
```

### Tier 1: Spike Us Back — $19+ (pay what you feel)

```
You've been spiking mockups. Now spike us.

💚 Love it — $29+
   "Spikes changed how I work with clients"
   
💙 Like it — $19
   "Solid tool, does what it says"
   
😐 It's fine — $9
   "Meh, but I respect the craft"

What you get:
- Our actual gratitude (not a marketing line)
- Supporter badge for your site
- Priority on GitHub issues (tag with [SUPPORTER])
- Name on the supporters page (if you want)

What you don't get:
- More features (you already have them all)
- Guilt if you don't pay (we mean it)
```

### Tier 2: Agency License — $149 once

```
For teams billing clients for design work.

- Covers your whole agency (honor system)
- Logo on spikes.sh
- Priority email support
- "Feedback powered by Spikes" badge for client deliverables
- Influence on roadmap
- Good karma (unquantifiable but real)
```

### Tier 3: Fill the Cup — Custom

```
Want to fund development directly?

Sponsor a feature. Fund a month of work.
Name a release. Whatever feels right.

Email us. We'll figure it out together.
```

---

## Why These Prices?

**$9 minimum** — Low enough to be impulse, high enough to filter noise.

**$19 suggested** — The price of a nice dinner. Not a commitment.

**$149 agency** — Agencies billing clients $50k can afford it. Signals legitimacy without being extractive. Honor system for seat count.

**No subscription** — Subscriptions exhaust everyone. Pay once, use forever.

---

## Payment Infrastructure: Stripe

### Why Stripe Direct
- You already have it
- 2.9% + $0.30 per transaction (vs 5-10% on Gumroad/Lemon Squeezy)
- Native "pay what you want" via Payment Links
- Full control over customer data
- Webhook support for automation

### Setup

1. **Create Products**
   ```
   "Spikes Supporter" — Customer chooses amount, min $9, suggested $19
   "Spikes Agency" — Fixed $149
   ```

2. **Create Payment Links**
   - Enable "Customer chooses amount" for Supporter
   - Add custom field: "How should we list you?" (Public name / Anonymous / Don't list)

3. **Success Flow**
   - Redirect to /thank-you with badge instructions
   - Stripe sends receipt automatically
   - Webhook updates supporters.json (optional automation)

---

## Badge Implementation

### Approach: Honor System + Light Verification

No DRM. No license servers. Trust people.

**Badge HTML:**
```html
<a href="https://spikes.sh/supporters#[hash]">
  <img src="https://spikes.sh/badge-supporter.svg" alt="Spikes Supporter" />
</a>
```

**Verification:**
- Hash = first 8 chars of SHA256(email)
- Supporters page lists hashes
- Anyone can verify a badge links to a real entry
- No emails exposed

**Badge Variants:**
- `badge-supporter.svg` — Red, "Spikes Supporter"
- `badge-agency.svg` — Blue, "Spikes Agency"
- `badge-love.svg` — Green heart, for 💚 supporters

---

## Supporters Page Structure

```
/supporters

# Supporters

These people keep Spikes alive.

## Agencies
[Logo] Studio Name
[Logo] Agency Name

## Supporters
(147 people as of Feb 2026)

Public names appear here.
Anonymous supporters show as hashes: abc123..., def456...

---

Spike us back?
[💚 Love $29+] [💙 Like $19] [😐 Fine $9] [Agency $149]
```

---

## Email Templates

### Supporter Confirmation

```
Subject: You spiked us back 💚

Hey [name],

Thanks for supporting Spikes with $[amount]. 
The cup is a little more full.

YOUR BADGE
Add this to your site:

<a href="https://spikes.sh/supporters#[hash]">
  <img src="https://spikes.sh/badge-supporter.svg" alt="Spikes Supporter" />
</a>

PRIORITY ISSUES
Tag your GitHub issues with [SUPPORTER] — we'll prioritize them.

SUPPORTERS PAGE
You're at: spikes.sh/supporters#[hash]
Want your name shown? Reply to this email.

— Moritz
spikes.sh
```

### Agency Confirmation

```
Subject: Spikes Agency License

Hey [name],

Thanks for the agency license. Your team is covered.

LICENSE ID: SPIKES-AGENCY-[hash]
This is honor system. No enforcement, no tracking.

YOUR BADGE
<a href="https://spikes.sh/supporters#[hash]">
  <img src="https://spikes.sh/badge-agency.svg" alt="Spikes Agency" />
</a>

LOGO ON SPIKES.SH
Reply with your logo (SVG preferred, 200x50 max).

PRIORITY SUPPORT
Email support@spikes.sh with [AGENCY] in subject.

— Moritz
```

---

## FAQ

### Is the free tier actually free?
Yes. 100% of features, forever. No trial, no "free for personal use," no asterisks.

### Why would I pay then?
Because you want indie tools to exist. Or because you want priority support and a badge. Or because Spikes saved you time and you want to say thanks.

### What's the minimum I can pay?
$9 for Supporter. But we suggest $19 if Spikes earns its place in your workflow.

### Can I pay more?
Yes. The checkout lets you enter any amount. Some people pay $50, $100, whatever feels right. We appreciate every dollar.

### Is Agency really $149 once?
Yes. One payment, whole team, forever. No renewals. If your team is 50 people using one $149 license... that's between you and your conscience.

### How do you verify team size?
We don't. Honor system.

### What if I need an invoice?
Stripe provides invoices automatically.

### Can I get a refund?
Within 30 days, no questions asked.

### Why not subscription?
Subscriptions are exhausting. Pay once, use forever. If we build Spikes 2.0 someday, that might be a separate purchase.

---

## Revenue Expectations

Be realistic:
- **Month 1:** 5-20 supporters ($100-400)
- **Month 6:** 50-100 total ($1,000-2,000 cumulative)
- **Year 1:** Maybe $5,000 if the tool gets traction

This is a tip jar for a side project. The model works because:
- Near-zero infrastructure (you deploy to YOUR Cloudflare)
- Near-zero support burden (it's simple software)
- Revenue from people who *want* to pay, not people forced to

---

## The Vibe

Channel the Four of Swords. Peace. Quiet confidence.

Don't:
- Apologize for being free
- Beg for support
- Add "MOST POPULAR" badges
- Create urgency ("limited time!")

Do:
- Be generous
- Trust people
- Make payment feel like celebration
- Keep the cup full enough to keep building
