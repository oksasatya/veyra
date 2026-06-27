# Veyra Design System

A small, sync-ready brand bundle for Veyra — the source for the Claude Design
(`claude.ai/design`) project and the reference for the Flutter app's theme.

## What's here

```
design-system/
├── tokens.json              # brand tokens (reference -> semantic): colors, fonts, radii
├── assets/
│   ├── logo-mark.svg        # V monogram (icon / favicon)
│   └── logo-lockup.svg      # mark + "veyra" wordmark
├── foundations/
│   ├── colors.html          # palette swatches (hex + OKLCH + token)
│   ├── typography.html      # type scale (Sora / IBM Plex Sans / IBM Plex Mono)
│   └── logo.html            # logo on dark/light, mark, monochrome
└── components/
    ├── button.html          # filled / ghost / disabled, two sizes
    ├── text-field.html      # default / focused / error / disabled
    └── card.html            # vehicle summary card
```

Each preview HTML starts with a `<!-- @dsCard group="..." name="..." -->` marker, which the
Claude Design pane uses to build its card index automatically.

## Brand at a glance

- **Palette:** Carbon + Signal Amber (dark-first). Accent `#F26A21`, background `#0D1119`.
- **Type:** Sora (display) · IBM Plex Sans (body) · IBM Plex Mono (data/hex).
- **Logo:** layered V monogram + lowercase wordmark.

Full values live in `tokens.json` and `foundations/colors.html`.

## Syncing to Claude Design

The `DesignSync` tool needs design-system authorization, which requires an interactive terminal
(`/design-login`) — it cannot run from a headless/web session. To push this bundle:

1. Open this repo in an interactive `claude` terminal (or use Claude Design's
   "Send to Claude Code Web" to seed the project into the workspace).
2. Run `/design-sync` and point it at this `design-system/` directory.
3. Approve the plan; the foundation + component cards upload to the Design System pane.

The sync is incremental (one component at a time), never a wholesale replace.
