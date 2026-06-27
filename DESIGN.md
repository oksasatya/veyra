# Veyra — Design System

Canonical tokens: `design-system/tokens.json`. Cards: `design-system/` (foundations + components +
screens). Register: **product** (Restrained color, earned familiarity, state-rich components).

## Color — Carbon + Signal Amber (dark-first)

| Role | Token | Hex | OKLCH |
|---|---|---|---|
| Background | `--bg` | `#0D1119` | `oklch(16% .015 255)` |
| Surface | `--surface` | `#151A23` | `oklch(22% .015 250)` |
| Surface raised | `--surface-2` | `#1B212C` | `oklch(25% .015 250)` |
| Border | `--border` | `#232B36` | `oklch(30% .020 250)` |
| Text | `--text` | `#E6EAF0` | `oklch(93% .008 250)` |
| Text muted | `--text-muted` | `#9BA6B5` | `oklch(72% .020 250)` |
| Accent (signature) | `--accent` | `#F26A21` | `oklch(68% .18 45)` |
| Accent hover | `--accent-hover` | `#FF8338` | `oklch(74% .17 48)` |
| Info / data | `--info` | `#34D1C4` | `oklch(78% .12 185)` |
| Danger | `--danger` | `#F2555A` | `oklch(64% .19 22)` |

Restrained strategy: graphite surfaces carry the UI; amber is reserved for primary actions, the current
selection, and state indicators, never decoration. Contrast: body text `--text` on `--bg`/`--surface`
clears 4.5:1; never set body in `--text-muted` on a tinted surface below 4.5:1.

## Type

- **Display:** Sora (600/700) — screen titles, the wordmark. Used with restraint.
- **Body / UI:** IBM Plex Sans (400/500/600) — labels, buttons, body. The workhorse.
- **Data:** IBM Plex Mono (500) — odometer, money, plate numbers, hex.
- Fixed rem scale (product register), ratio ~1.2. No fluid clamp headings in UI.

## Motion

150–250 ms, ease-out. Motion conveys state (press, selection, sheet in/out, value change), never
decoration. No page-load choreography. Honor `prefers-reduced-motion`.

## Components

Every interactive element ships default / hover / focus / active / disabled / loading / error.
Skeletons for loading, teaching empty states. Radii: sm 10, md 12, lg 14, xl 18, pill 999.
Bans honored: no side-stripe borders, no gradient text, no default glassmorphism, no hero-metric
template, no identical card grids, no per-section uppercase eyebrows.

## Platform adaptation

iOS: large title, grouped surfaces, bottom tab bar, no FAB. Android: Material 3 top app bar, bottom
nav, FAB for the primary create action. One Flutter codebase; chrome adapts per platform.
