# UI Contracts: UI Layout Polish

**Feature**: 004-ui-layout-polish  
**Date**: 2026-02-27

## Overview

This document defines the target styling contracts for each component affected by the UI polish. No Tauri commands or backend changes are required. All changes are frontend-only CSS/Tailwind class modifications.

## Component Styling Contracts

### 1. Sidebar (`Sidebar.tsx`)

**Current**: Two conditional `<div>` branches; `w-12` (collapsed) and `w-64` (expanded) render different trees. No transition animation.

**Target**:
- Single persistent `<div>` container with class-toggled width.
- Width transition: `transition-[width] duration-300 ease-in-out`.
- Container: `overflow-hidden` to clip content during transition.
- Collapsed state: Icons only, text hidden via `opacity-0` and `w-0 overflow-hidden` on text elements.
- Expanded state: Text fades in with `animate-in fade-in duration-200`.
- Header: `px-4 py-3` padding, consistent border-b.
- Conversation items: `px-3 py-2.5` padding, `rounded-lg` for hover/active states.
- Conversation list: Use `<ScrollArea>` (shadcn/ui) instead of `overflow-y-auto`.
- Icon buttons: Use `<Tooltip>` (shadcn/ui) instead of native `title` attribute.
- Long titles: `truncate` class for single-line ellipsis.
- Delete button: `transition-opacity duration-150` (keep existing pattern).

**Spacing contract**:
| Element | Spacing |
|---------|---------|
| Sidebar header | `px-4 py-3` |
| Conversation items | `px-3 py-2.5 mx-2` |
| Items gap | `gap-1` |
| Collapsed icon buttons | `mx-auto` centered |

---

### 2. MainPanel - Chat Area (`ChatArea.tsx`)

**Current**: `px-4 py-4`, `max-w-3xl`, `gap-4` between messages.

**Target**:
- Container padding: `px-4 py-6` (slightly more vertical breathing room).
- Message gap: `gap-6` (24px) for comfortable reading rhythm.
- Content column: `max-w-3xl mx-auto` (keep existing centering).
- Loading indicator: Keep `flex gap-3` with `h-8 w-8` avatar, `rounded-2xl` bubble.

---

### 3. MessageBubble (`MessageBubble.tsx`)

**Current**: `gap-3`, `max-w-[75%]`, `px-3 py-2`, `h-16 w-16` thumbnails, `text-[8px]` badge.

**Target**:
- Row gap: `gap-3` (keep).
- Content max width: `max-w-[80%]` (slightly wider for image-heavy content).
- Text bubble padding: `px-4 py-3` (16px / 12px) for comfortable reading.
- Text size: `text-sm` (keep, 14px).
- Bubble rounding: `rounded-2xl` (softer, more modern look).
- Avatar: `h-8 w-8 rounded-full` (keep size, add subtle shadow `shadow-sm`).
- Attachment thumbnails: `h-20 w-20` (upgrade from 64px to 80px for better preview).
- Attachment gap: `gap-2` (upgrade from `gap-1.5`).
- Order badge: `text-xs` (replace `text-[8px]`), `h-5 w-5` (upgrade from `h-4 w-4`).
- Results gap: `gap-3` (upgrade from `gap-2`).

**Spacing contract**:
| Element | Current | Target |
|---------|---------|--------|
| Bubble padding | `px-3 py-2` | `px-4 py-3` |
| Bubble rounding | `rounded-lg` | `rounded-2xl` |
| Attachment size | `h-16 w-16` | `h-20 w-20` |
| Attachment gap | `gap-1.5` | `gap-2` |
| Order badge font | `text-[8px]` | `text-xs` |
| Results gap | `gap-2` | `gap-3` |

---

### 4. ImageResult (`ImageResult.tsx`)

**Current**: `max-h-80`, `rounded-lg`, hover toolbar with `right-2 top-2`.

**Target**:
- Image container rounding: `rounded-xl` (softer).
- Max image height: `max-h-96` (upgrade from 320px to 384px for better detail viewing).
- Hover toolbar: `right-3 top-3` with `gap-1.5` (slightly more breathing room).
- Toolbar buttons: `h-8 w-8 rounded-lg` (softer rounding).
- Preview modal close button: Keep `right-4 top-4 h-10 w-10`.

---

### 5. Input Area (`MainPanel.tsx` bottom section)

**Current**: `border-t border-border px-4 py-3`, `space-y-2`.

**Target**:
- Container: `border-t border-border px-4 py-4` (slightly more vertical padding).
- Inner spacing: `space-y-3` (upgrade from 8px to 12px gap between rows).
- Top row (mode/model selector): `flex items-center justify-between` (keep).
- Max width: `max-w-3xl mx-auto` (keep, matches chat area).
- Translate button: `rounded-xl` (softer), `py-2.5` (taller click target).

---

### 6. PromptInput (`PromptInput.tsx`)

**Current**: `gap-2`, `rounded-lg`, inline styles for min/max height.

**Target**:
- Container gap: `gap-3` (upgrade from 8px to 12px).
- Textarea rounding: `rounded-xl` (softer).
- Textarea padding: `px-4 py-3` (upgrade from `px-3 py-2`).
- Send button: `h-10 w-10 rounded-xl` (softer rounding).
- Min height: 44px (upgrade from 40px via `min-h-11`).
- Max height: 200px (keep via inline style, no standard Tailwind class for this).

---

### 7. ModeSelector (`ModeSelector.tsx`)

**Current**: `p-1 rounded-lg gap-1`, tabs `px-3 py-1.5 text-xs`.

**Target**:
- Container: `p-1 rounded-xl gap-1` (softer rounding).
- Tab buttons: `px-3 py-1.5 text-xs rounded-lg` (keep size, softer rounding).
- Active tab: Add `shadow-sm` for subtle elevation.
- Icons: `h-3.5 w-3.5` (keep).

---

### 8. ImageUpload (`ImageUpload.tsx`)

**Current**: `h-20 w-20` thumbnails, `text-[10px]` order badge, `gap-2`.

**Target**:
- Thumbnails: `h-24 w-24` (upgrade from 80px to 96px for better preview).
- Thumbnail rounding: `rounded-xl` (softer).
- Order badge: `text-xs` (replace `text-[10px]`), `h-5 w-5`.
- Add button: `h-24 w-24 rounded-xl border-2 border-dashed`.
- Grid gap: `gap-2.5` (upgrade from 8px to 10px).

---

### 9. LanguageSelector (`LanguageSelector.tsx`)

**Current**: `gap-2`, selects with `rounded-md`, swap button `h-8 w-8 rounded-md`.

**Target**:
- Gap: `gap-3` (upgrade from 8px to 12px).
- Select rounding: `rounded-lg` (softer).
- Swap button: `h-8 w-8 rounded-lg` (softer rounding).

---

### 10. Welcome Screen (`MainPanel.tsx` welcome section)

**Current**: `gap-4 p-8`, cards `p-3`, card text `text-xs` / `text-[10px]`.

**Target**:
- Container: `gap-6 p-8` (more spacing between elements).
- Title: `text-2xl font-semibold` (keep).
- Subtitle: `mt-3 text-sm` (upgrade `mt-2` to `mt-3`).
- Feature cards: `p-5 rounded-xl gap-4` (bigger padding, softer rounding).
- Card grid: `grid max-w-xl grid-cols-3 gap-5` (wider max-width, bigger gaps).
- Card icon: (if present) centered, `text-muted-foreground`.
- Card title: `text-sm font-medium` (upgrade from `text-xs`).
- Card description: `text-xs text-muted-foreground mt-1.5` (replace `text-[10px]`).

---

### 11. ErrorDisplay (`ErrorDisplay.tsx`)

**Current**: Hardcoded `border-red-200 bg-red-50 text-red-800` with `dark:` overrides.

**Target**:
- Container: `border-destructive/20 bg-destructive/5 text-foreground`.
- Icon color: `text-destructive`.
- Error detail text: `text-destructive/80`.
- Remove all `dark:` prefixes — design tokens handle mode switching.
- Rounding: `rounded-xl` (upgrade from `rounded-lg`).

---

### 12. ModelSelector (`ModelSelector.tsx`)

**Current**: `rounded-md`, `text-xs`.

**Target**:
- Rounding: `rounded-lg`.
- Keep `text-xs` size (appropriate for a secondary control).

---

## New Dependencies

| Component | Package | Installation |
|-----------|---------|-------------|
| Tooltip | `@radix-ui/react-tooltip` (via shadcn) | `npx shadcn add tooltip` |
| ScrollArea | `@radix-ui/react-scroll-area` (via shadcn) | `npx shadcn add scroll-area` |

## Global Changes

### cn() Utility Adoption
All components currently using raw template string concatenation for conditional classes (e.g., `` `${condition ? 'class-a' : 'class-b'}` ``) should migrate to `cn()` from `@/lib/utils`. This improves:
- Class conflict resolution (via `tailwind-merge`)
- Readability of conditional logic
- Consistency across components

### TooltipProvider
`App.tsx` must wrap the main content with `<TooltipProvider>` to enable all `<Tooltip>` instances.
