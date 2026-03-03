# Widget Attributes Reference

The Spikes widget is configured via `data-*` attributes on the script tag. This reference covers all supported attributes.

## Basic Usage

```html
<script src="https://spikes.sh/spikes.js"
  data-project="my-app"
  data-position="bottom-right"
  data-color="#e74c3c">
</script>
```

---

## Attributes

### data-project

The project identifier for grouping feedback. Spikes are stored per-project in localStorage and tagged with this key when synced to a backend.

| Property | Value |
|----------|-------|
| **Description** | Project key for organizing spikes |
| **Valid Values** | Any string (letters, numbers, hyphens, underscores) |
| **Default Value** | `location.hostname` or `"local"` |
| **Example** | `data-project="acme-dashboard"` |

```html
<script src="spikes.js" data-project="acme-dashboard"></script>
```

---

### data-position

Position of the feedback button in the viewport. The button is fixed in the specified corner.

| Property | Value |
|----------|-------|
| **Description** | Corner position for the feedback button |
| **Valid Values** | `"bottom-right"`, `"bottom-left"`, `"top-right"`, `"top-left"` |
| **Default Value** | `"bottom-right"` |
| **Example** | `data-position="top-left"` |

```html
<script src="spikes.js" data-position="top-left"></script>
```

---

### data-color

Accent color for the feedback button and UI highlights. Accepts any valid CSS color value.

| Property | Value |
|----------|-------|
| **Description** | Accent color for button and UI elements |
| **Valid Values** | CSS color value (hex, rgb, named colors) |
| **Default Value** | `"#e74c3c"` (Spikes red) |
| **Example** | `data-color="#3b82f6"` |

```html
<script src="spikes.js" data-color="#3b82f6"></script>
<script src="spikes.js" data-color="rgb(59, 130, 246)"></script>
```

---

### data-theme

Visual theme for the feedback modal and popover. Controls background colors and text colors.

| Property | Value |
|----------|-------|
| **Description** | Visual theme for modal/popover UI |
| **Valid Values** | `"dark"`, `"light"` |
| **Default Value** | `"dark"` |
| **Example** | `data-theme="light"` |

```html
<script src="spikes.js" data-theme="light"></script>
```

---

### data-reviewer

Pre-sets the reviewer name, skipping the name prompt. Useful for embedding in known contexts where the reviewer identity is already established.

| Property | Value |
|----------|-------|
| **Description** | Pre-set reviewer name (skips name prompt) |
| **Valid Values** | Any non-empty string |
| **Default Value** | `null` (prompts for name on first spike) |
| **Example** | `data-reviewer="Pat"` |

```html
<script src="spikes.js" data-reviewer="Pat"></script>
```

---

### data-endpoint

API endpoint URL for syncing spikes to a backend. If set, spikes are POSTed to this URL immediately after local save.

| Property | Value |
|----------|-------|
| **Description** | Backend API URL for spike sync |
| **Valid Values** | Valid HTTPS URL (or `http://` for local dev) |
| **Default Value** | `null` (posts to `/spikes` if served via HTTP) |
| **Example** | `data-endpoint="https://spikes.sh/api/spikes?token=abc123"` |

```html
<script src="spikes.js" 
  data-endpoint="https://spikes.sh/api/spikes?token=abc123">
</script>
```

---

### data-collect-email

Enables an optional email field in the reviewer name prompt. Email is stored with the spike for follow-up notifications.

| Property | Value |
|----------|-------|
| **Description** | Show email field in reviewer prompt |
| **Valid Values** | `"true"` (any other value is treated as false) |
| **Default Value** | `false` |
| **Example** | `data-collect-email="true"` |

```html
<script src="spikes.js" data-collect-email="true"></script>
```

---

### data-admin

Enables admin features in the widget UI. Currently shows a "Review" button that toggles review mode, displaying spike markers on the page.

| Property | Value |
|----------|-------|
| **Description** | Enable admin/review mode features |
| **Valid Values** | `"true"` (any other value is treated as false) |
| **Default Value** | `false` |
| **Example** | `data-admin="true"` |

```html
<script src="spikes.js" data-admin="true"></script>
```

---

### data-offset-x

Horizontal offset from the configured position. Positive values move the button toward the center. Accepts CSS length values.

| Property | Value |
|----------|-------|
| **Description** | Horizontal offset from edge |
| **Valid Values** | CSS length (e.g., `"20px"`, `"2rem"`, `"10"`) |
| **Default Value** | `null` (no offset) |
| **Example** | `data-offset-x="50px"` |

```html
<script src="spikes.js" 
  data-position="bottom-right"
  data-offset-x="50px">
</script>
```

---

### data-offset-y

Vertical offset from the configured position. Positive values move the button toward the center. Accepts CSS length values.

| Property | Value |
|----------|-------|
| **Description** | Vertical offset from edge |
| **Valid Values** | CSS length (e.g., `"20px"`, `"2rem"`, `"10"`) |
| **Default Value** | `null` (no offset) |
| **Example** | `data-offset-y="30px"` |

```html
<script src="spikes.js" 
  data-position="bottom-right"
  data-offset-y="30px">
</script>
```

---

## Combining Attributes

All attributes can be combined. Here's a complete example:

```html
<script 
  src="https://spikes.sh/spikes.js"
  data-project="my-app"
  data-position="bottom-right"
  data-color="#3b82f6"
  data-theme="dark"
  data-reviewer="Design Team"
  data-endpoint="https://api.example.com/spikes?token=secret"
  data-collect-email="true"
  data-admin="true"
  data-offset-x="20px"
  data-offset-y="20px">
</script>
```

## Widget z-index

All widget elements use `z-index: 2147483647` (maximum 32-bit signed integer) to ensure they render above all page content. This includes:

- Feedback button (`#spikes-btn`)
- Modal dialog (`#spikes-modal`)
- Element popover (`#spikes-popover`)
- Review markers (`.spikes-review-marker`)
- Toast notifications (`#spikes-toast`)
