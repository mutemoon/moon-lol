# Design

## Theme

Dark, warm, premium. The interface lives in the shadows of Runeterra — deep brown-blacks, aged parchment, and luminous gold. Light is scarce and purposeful. The app feels like a physical object: leather, metal, stone, old paper.

> A player at a dimly lit desk, evening session, single monitor. The app glows warm gold against the dark room. It belongs on a gaming desktop, not a phone or a browser.

## Color

All values in OKLCH. Chroma held below 0.03 near extremes (L < 10 or L > 95) to avoid garish edges.

### Neutral palette

| Token | OKLCH | Hex (approx) | Role |
|---|---|---|---|
| `--bg-deep` | oklch(4% 0.008 65) | `#070608` | Deepest background, body |
| `--bg-surface` | oklch(9% 0.012 60) | `#121013` | Cards, panels, dropdowns |
| `--bg-elevated` | oklch(13% 0.015 58) | `#1c1820` | Hovered surfaces, active inputs |
| `--bg-raised` | oklch(18% 0.018 55) | `#292231` | Raised elements, modal overlays |
| `--border-subtle` | oklch(22% 0.02 55) | `#352c3d` | Default borders, dividers |
| `--border-default` | oklch(28% 0.025 55) | `#443a50` | Input borders, card borders |
| `--text-muted` | oklch(45% 0.02 70) | `#685e5a` | Secondary text, placeholders |
| `--text-default` | oklch(65% 0.025 80) | `#9a9282` | Body text |
| `--text-bright` | oklch(88% 0.03 85) | `#dbd6c5` | Primary text, headings |

### Gold accent palette

| Token | OKLCH | Hex (approx) | Role |
|---|---|---|---|
| `--gold-dimmer` | oklch(45% 0.08 75) | `#785b28` | Disabled gold, shadow gold |
| `--gold-muted` | oklch(55% 0.10 75) | `#927136` | Border base, inactive gold |
| `--gold-default` | oklch(65% 0.13 78) | `#b99147` | Primary gold, active borders |
| `--gold-bright` | oklch(73% 0.14 80) | `#d4af5c` | Hover gold, gold text |
| `--gold-glow` | oklch(80% 0.12 82) | `#e8c97a` | Glow effects, luminous accents |

### Semantic

| Token | OKLCH | Hex (approx) | Role |
|---|---|---|---|
| `--red` | oklch(55% 0.18 25) | `#c84a4a` | Errors, destructive actions |
| `--green` | oklch(55% 0.14 145) | `#4a9e5a` | Success, connected status |
| `--blue` | oklch(55% 0.10 240) | `#4a7ec4` | Info, links |
| `--cyan` | oklch(55% 0.08 200) | `#4a9e9e` | Alternative accent |

### Application

- **Body background**: `--bg-deep` with a faint radial gradient (`--gold-dimmer` at 0%, blended at 3% opacity, center-top) to give the dark warmth
- **Surface backgrounds**: `--bg-surface` for cards, `--bg-elevated` for hover
- **Borders**: `--border-subtle` default, `--gold-muted` for interactive component borders
- **Primary text**: `--text-bright` for headings, `--text-default` for body
- **Gold usage**: buttons, active states, navigation indicators, decorative dividers

## Typography

### Font stack

```css
--font-display: "Noto Serif SC", "Noto Serif", Georgia, serif;
--font-body: "Noto Sans SC", "Noto Sans", "Segoe UI", Arial, sans-serif;
--font-mono: "Cascadia Code", "JetBrains Mono", "Fira Code", Consolas, monospace;
```

Serif for titles and display text (warmth, weight, Runeterra). Sans for body and UI (readability). Mono for log output and data displays.

### Scale

| Step | Size | Weight | Line Height | Font | Use |
|---|---|---|---|---|---|
| `--fs-display` | 40px / 2.5rem | 700 | 1.15 | display | App title, splash |
| `--fs-h1` | 28px / 1.75rem | 700 | 1.2 | display | Section headings |
| `--fs-h2` | 20px / 1.25rem | 600 | 1.25 | body | Panel titles |
| `--fs-h3` | 16px / 1rem | 600 | 1.3 | body | Card titles, nav labels |
| `--fs-body` | 14px / 0.875rem | 400 | 1.5 | body | Body text, controls |
| `--fs-small` | 12px / 0.75rem | 400 | 1.5 | body | Labels, metadata |
| `--fs-tiny` | 11px / 0.6875rem | 500 | 1.4 | body | Badges, timestamps |
| `--fs-mono` | 13px / 0.8125rem | 400 | 1.5 | mono | Log output, code |

### Letter spacing

- Display text: `0.02em`
- Gold accent text (nav, buttons, premium labels): `0.06em` (uppercase)
- Body: normal

## Elevation & Shadows

Shadows use a warm-brown tint (oklch(0% 0 60)) rather than neutral black for cohesion with the dark warm background.

| Token | Value | Use |
|---|---|---|
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.4), 0 0 1px rgba(120,91,40,0.15)` | Subtle surface separation |
| `--shadow-md` | `0 4px 12px rgba(0,0,0,0.5), 0 0 2px rgba(120,91,40,0.2)` | Cards, dropdowns |
| `--shadow-lg` | `0 12px 32px rgba(0,0,0,0.6), 0 0 4px rgba(120,91,40,0.15)` | Modals, overlays |
| `--shadow-glow-gold` | `0 0 12px rgba(201,170,113,0.25), 0 0 4px rgba(201,170,113,0.4)` | Gold glow on active elements |
| `--shadow-inner-gold` | `inset 0 1px 0 rgba(201,170,113,0.15)` | Inner top border highlight on gold elements |
| `--shadow-inner-deep` | `inset 0 2px 4px rgba(0,0,0,0.5)` | Deep inner shadow for inputs |

## Border

| Token | Value | Use |
|---|---|---|
| `--radius-sm` | 3px | Small badges, inline elements |
| `--radius-md` | 6px | Inputs, buttons, small cards |
| `--radius-lg` | 10px | Panels, modal content |
| `--radius-xl` | 16px | Large containers |

Gold borders use a gradient technique:

```
border: 1px solid transparent;
border-image: linear-gradient(135deg, var(--gold-dimmer), var(--gold-default), var(--gold-bright)) 1;
```

For simpler cases, a solid `--gold-muted` or `--gold-default` border suffices.

## Components

### Button (Primary)

- Background: `--bg-surface`
- Border: gold gradient, 1px
- Text: `--gold-bright`, uppercase, `0.06em` letter-spacing, `--fs-small`
- Padding: 8px 24px (horizontal), 10px (vertical)
- Border-radius: `--radius-md`
- Inner shadow: `--shadow-inner-gold`
- Box shadow: `--shadow-sm`

States:
- **Hover**: background → `--bg-elevated`, text → `--gold-glow`, `--shadow-glow-gold`, text-shadow: `0 0 6px rgba(232,201,122,0.4)`
- **Active/Pressed**: scale 0.97, brightness 1.1, quick shine sweep animation
- **Disabled**: `--gold-dimmer` border, `--text-muted` text, no glow, no hover effects

### Button (Ghost)

- Background: transparent
- Border: `--border-subtle`, 1px solid
- Text: `--text-default`
- States: hover border → `--gold-muted`, text → `--gold-bright`

### Button (Danger)

- Background: `--bg-surface`
- Border: `--red`, 1px solid
- Text: `--red`
- States: hover background → `rgba(200,74,74,0.1)`, glow shadow in red

### Input

- Background: `--bg-deep` with `--shadow-inner-deep`
- Border: `--gold-dimmer`, 1px solid
- Text: `--text-bright`
- Placeholder: `--text-muted`
- Padding: 10px 14px
- Border-radius: `--radius-md`
- Focus: border → `--gold-default`, box-shadow `--shadow-glow-gold`

### Select (Dropdown)

Same as Input. The chevron icon uses a gold tint.

### Card

- Background: `--bg-surface`
- Border: `--border-subtle`, 1px solid
- Border-radius: `--radius-lg`
- Padding: 20px
- Box shadow: `--shadow-md`
- Hover: border → `--gold-muted`, box-shadow → `--shadow-lg`

### Badge / Tag

- Background: `--bg-elevated`
- Border: `--border-subtle`
- Border-radius: `--radius-sm`
- Text: `--text-small`, uppercase, `0.04em`
- Padding: 2px 8px

### Modal

- Overlay: `rgba(0,0,0,0.75)` with backdrop-filter blur(4px)
- Content: `--bg-surface`, border → gold gradient, `--radius-xl`, `--shadow-lg`
- Header: decorative gold divider line at bottom
- Close button: ghost style, top-right

### Status Indicator (Dot)

- Connected: `--green`, box-shadow `0 0 8px rgba(74,158,90,0.5)`
- Disconnected: `--red`, box-shadow `0 0 8px rgba(200,74,74,0.4)`
- Size: 8px diameter

### Navigation Tab

- Inactive: `--text-muted`, no border
- Active: `--gold-bright` text, gold bottom border (2px, `--gold-default`), or gold triangular indicator
- Hover: `--text-default` with subtle gold text-shadow
- Padding: 8px 16px, uppercase, `0.06em` letter-spacing

## Layout

### Shell

- Full-height window, no title bar chrome (Tauri decorations or custom titlebar)
- Vertical layout: top nav bar (60-76px) + scrollable content area
- Content max-width: 1200px, centered with padding
- No sidebar — navigation is horizontal tabs

### Nav Bar

- Fixed position, top
- Background: `rgba(7,6,8,0.85)` with `backdrop-filter: blur(40px)`
- Bottom border: `--border-subtle`, 1px
- Height: 64px
- Left: app logo/title
- Center/Right: navigation tabs
- Right edge: action buttons (settings, minimize, close)

### Spacing

- Surface padding: 24px (panel), 20px (card), 16px (compact card)
- Between children: 16px (sections), 12px (related controls), 8px (tight groups)
- Grid gap for card layouts: 16px

Use a 4px base unit. Spacing values: 4, 8, 12, 16, 20, 24, 32, 40, 48, 64.

## Motion

### Timing

| Token | Duration | Easing | Use |
|---|---|---|---|
| `--dur-instant` | 0.1s | ease-out | Micro feedback, hover states |
| `--dur-fast` | 0.2s | ease-out | Button active, status changes |
| `--dur-normal` | 0.3s | ease-out | Transitions, panel toggles |
| `--dur-slow` | 0.5s | ease-out | Page transitions, modal open |
| `--dur-reveal` | 0.8s | ease-out | Initial load animations |

Easing curve for custom animations: `cubic-bezier(0.16, 1, 0.3, 1)` — exponential ease-out feel.

### Animations

- **Fade in**: opacity 0 → 1, `--dur-normal`
- **Fade up**: translateY(8px) → translateY(0), opacity 0 → 1, `--dur-slow`
- **Gold shimmer**: gradient sweep on hover/active, 0.6s, used on primary buttons
- **Pulse glow**: box-shadow oscillates between `--shadow-glow-gold` (+4px spread) and base, 2s cycle, used on status indicators and active toggles
- **Scale press**: transform scale(1) → scale(0.97), `--dur-instant`, on button mousedown
- **Spin**: 0.8s linear infinite, for loading spinners

### Reduced Motion

When `prefers-reduced-motion: reduce`:
- Remove shimmer, glow pulse, and decorative animations
- Keep opacity transitions at `--dur-fast` (functional, not decorative)
- Keep scale press (functional feedback)
- Remove fade-up in favor of instant appearance

## Icons

Use simple SVG icons with `currentColor` so they inherit text color. Gold for interactive icons, `--text-muted` for decorative. 16x16 default size for inline icons, 20x20 for nav icons, 24x24 for action icons.

## Texture

Subtle noise/ grain texture on the deepest background layer adds physical depth. Applied as a CSS pseudo-element with a very low opacity PNG/SVG noise pattern blended via `mix-blend-mode: overlay` at 3-5% opacity.

```
.bg-texture::after {
  content: '';
  position: fixed;
  inset: 0;
  background-image: url('/texture-noise.png');
  opacity: 0.03;
  mix-blend-mode: overlay;
  pointer-events: none;
  z-index: 0;
}
```
