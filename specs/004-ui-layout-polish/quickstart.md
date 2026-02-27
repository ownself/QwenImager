# Quickstart: UI Layout Polish

**Feature**: 004-ui-layout-polish  
**Date**: 2026-02-27

## Prerequisites

- Node.js and pnpm installed
- Rust stable toolchain (for Tauri)
- Project dependencies installed (`pnpm install`)

## Setup

```bash
# Switch to feature branch
git checkout 004-ui-layout-polish

# Install shadcn/ui components (Tooltip + ScrollArea)
npx shadcn add tooltip scroll-area

# Verify installation
ls src/components/ui/
# Should show: tooltip.tsx, scroll-area.tsx
```

## Development Workflow

```bash
# Start Tauri dev server (frontend + backend)
cargo tauri dev

# Frontend-only type checking (no Rust compile)
npx tsc --noEmit

# Frontend-only build check
npx vite build
```

## What Changes

This feature modifies **frontend files only** — no Rust backend changes.

### Files to Modify

| File | Changes |
|------|---------|
| `src/App.tsx` | Add `<TooltipProvider>` wrapper |
| `src/components/layout/Sidebar.tsx` | Refactor to single-div, add transition animation, use Tooltip + ScrollArea |
| `src/components/layout/MainPanel.tsx` | Polish welcome cards, input area spacing |
| `src/components/chat/ChatArea.tsx` | Adjust message gap and padding |
| `src/components/chat/MessageBubble.tsx` | Polish bubble padding, rounding, thumbnail sizes |
| `src/components/chat/ImageResult.tsx` | Adjust image sizing and rounding |
| `src/components/input/PromptInput.tsx` | Polish input rounding and spacing |
| `src/components/input/ModeSelector.tsx` | Softer rounding |
| `src/components/input/ModelSelector.tsx` | Softer rounding |
| `src/components/input/ImageUpload.tsx` | Bigger thumbnails, fix badge font sizes |
| `src/components/input/LanguageSelector.tsx` | Adjust spacing and rounding |
| `src/components/common/ErrorDisplay.tsx` | Migrate to design tokens |

### New Files (from shadcn)

| File | Source |
|------|--------|
| `src/components/ui/tooltip.tsx` | `npx shadcn add tooltip` |
| `src/components/ui/scroll-area.tsx` | `npx shadcn add scroll-area` |

## Verification

After each component change:

1. **Visual check**: Run `cargo tauri dev` and verify the component looks polished
2. **Type check**: `npx tsc --noEmit` — 0 errors
3. **Build check**: `npx vite build` — success
4. **Dark mode**: Add `class="dark"` to `<html>` in `index.html` temporarily, verify all changes look correct in dark mode, then remove

## Key Design Decisions

- **Spacing scale**: All spacing uses Tailwind's 4px-based scale (no arbitrary pixel values)
- **Rounding**: Most containers upgraded from `rounded-lg` to `rounded-xl` or `rounded-2xl` for a softer, modern look
- **Transitions**: Sidebar uses `transition-[width] duration-300`, content uses `tw-animate-css` fade-in
- **Colors**: ErrorDisplay migrates from hardcoded reds to `destructive` design tokens
- **cn() utility**: All conditional class logic migrates to `cn()` from `@/lib/utils`

## Reference

- [Spec](spec.md) — Feature specification
- [Research](research.md) — Technical research and decisions
- [UI Contracts](contracts/ui-contracts.md) — Detailed per-component styling targets
