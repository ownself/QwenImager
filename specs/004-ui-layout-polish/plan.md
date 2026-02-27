# Implementation Plan: UI Layout Polish

**Branch**: `004-ui-layout-polish` | **Date**: 2026-02-27 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-ui-layout-polish/spec.md`

## Summary

Polish the application's frontend UI to eliminate inconsistent spacing, default-looking styling, and abrupt transitions. All changes are frontend-only — no Rust backend modifications. The approach is to systematically update each component's Tailwind classes following a defined spacing scale, adopt softer rounding for a modern aesthetic, add smooth sidebar transitions, install two shadcn/ui primitives (Tooltip, ScrollArea), and migrate hardcoded color values to the existing design token system.

## Technical Context

**Language/Version**: TypeScript 5.x + React 19  
**Primary Dependencies**: Tailwind CSS v4, shadcn/ui (Tooltip + ScrollArea), tw-animate-css, lucide-react, @dnd-kit  
**Storage**: N/A (no data changes)  
**Testing**: `npx tsc --noEmit` (type check), `npx vite build` (build check), visual verification  
**Target Platform**: Desktop (Windows/macOS/Linux via Tauri v2)  
**Project Type**: Desktop app (Tauri v2 + React frontend)  
**Performance Goals**: Sidebar transition completes within 300ms, no layout shift during animations  
**Constraints**: Zero new third-party UI libraries beyond shadcn/ui components (which use already-installed Radix dependencies). Frontend-only changes. Preserve component hierarchy and dark mode support.  
**Scale/Scope**: 12 component files modified, 2 new shadcn/ui files scaffolded

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution (`constitution.md`) contains only placeholder content (no actual principles defined). No gating constraints to enforce. PASS.

**Post-design re-check**: Still PASS. No constitution violations — all changes are frontend styling only.

## Project Structure

### Documentation (this feature)

```text
specs/004-ui-layout-polish/
├── plan.md              # This file
├── research.md          # Phase 0: Technical research and decisions
├── data-model.md        # Phase 1: No data model changes
├── quickstart.md        # Phase 1: Developer quick-start guide
├── contracts/
│   └── ui-contracts.md  # Phase 1: Per-component styling targets
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (affected files)

```text
src/
├── App.tsx                              # Add TooltipProvider wrapper
├── components/
│   ├── ui/                              # NEW: shadcn/ui primitives
│   │   ├── tooltip.tsx                  #   Styled tooltip component
│   │   └── scroll-area.tsx              #   Styled scrollbar component
│   ├── layout/
│   │   ├── Sidebar.tsx                  # Refactor + animation + Tooltip + ScrollArea
│   │   └── MainPanel.tsx                # Welcome cards + input area spacing
│   ├── chat/
│   │   ├── ChatArea.tsx                 # Message gap + padding
│   │   ├── MessageBubble.tsx            # Bubble styling + thumbnail sizes
│   │   └── ImageResult.tsx              # Image sizing + rounding
│   ├── input/
│   │   ├── PromptInput.tsx              # Input rounding + spacing
│   │   ├── ModeSelector.tsx             # Rounding
│   │   ├── ModelSelector.tsx            # Rounding
│   │   ├── ImageUpload.tsx              # Thumbnail sizes + badge fonts
│   │   └── LanguageSelector.tsx         # Spacing + rounding
│   └── common/
│       └── ErrorDisplay.tsx             # Design token migration
└── lib/
    └── utils.ts                         # cn() utility (already exists, no changes)
```

**Structure Decision**: This feature modifies existing files in-place. The only new files are the two shadcn/ui components scaffolded into `src/components/ui/`. No structural reorganization.

## Complexity Tracking

No constitution violations to justify. All changes are straightforward styling modifications within existing component boundaries.
