# Research: UI Layout Polish

**Feature**: 004-ui-layout-polish  
**Date**: 2026-02-27

## R1: Sidebar Width Animation Approach

### Decision
Use Tailwind CSS `transition-[width] duration-300 ease-in-out` on a single persistent `<div>`, toggling between `w-12` and `w-64` classes. Inner content fades in/out with `tw-animate-css` utilities.

### Rationale
The current sidebar renders two completely separate `<div>` trees via conditional rendering (`if collapsed return A; else return B;`). CSS transitions cannot work across conditional renders — they require a single DOM element whose class changes. Refactoring to a single `<div>` with class toggling enables smooth CSS width transitions with zero JavaScript animation libraries.

### Alternatives Considered
1. **CSS `@keyframes` animation**: More control but overly complex for a simple width change. Keyframes don't compose well with the class-toggle pattern.
2. **JavaScript animation library (framer-motion, react-spring)**: Introduces a new dependency. The spec explicitly forbids new libraries. Tailwind's built-in transitions are sufficient.
3. **`transition-all`**: Works but transitions every CSS property, which can cause unintended visual artifacts and minor performance overhead. `transition-[width]` is more precise.

### Implementation Notes
- `overflow-hidden` is required on the sidebar container during the transition to prevent content overflow.
- Inner sidebar content (text labels, conversation list) should use opacity/fade transitions with `tw-animate-css` (`animate-in fade-in duration-200`) when the sidebar expands, and be hidden (`opacity-0` or conditional render) when collapsed.
- The sidebar must be refactored from two conditional branches to a single `<div>` with adaptive inner content.

---

## R2: shadcn/ui Component Selection

### Decision
Install and use **Tooltip** and **ScrollArea** shadcn/ui components. Defer Skeleton, Alert, Badge, and Separator to future work.

### Rationale
- **Tooltip**: The sidebar already has 4 `title="..."` attributes for browser-native tooltips. Replacing with shadcn's `<Tooltip>` provides styled, animated tooltips matching the design system. High visual impact, minimal structural change.
- **ScrollArea**: The conversation list uses plain `overflow-y-auto` with the browser's default scrollbar, which looks unpolished. `<ScrollArea>` provides a thin, styled scrollbar that matches the neutral design language. This is a drop-in replacement.
- **Why not Skeleton/Alert**: These would change the error display and loading state structure, which the spec says to preserve. They can be added later if desired.

### Alternatives Considered
1. **Custom tooltip with CSS only**: Possible but requires significant custom styling work. shadcn's Tooltip is already configured for the project and handles positioning, delays, and accessibility.
2. **Custom scrollbar via CSS**: Webkit-only scrollbar styling (`::-webkit-scrollbar`) doesn't work cross-browser. Radix ScrollArea works everywhere.
3. **No shadcn/ui components**: Would leave browser-native tooltips and scrollbars, which look generic. The polish spec explicitly asks for a professional appearance.

### Implementation Notes
- `Tooltip` requires `<TooltipProvider>` wrapper at the app root (App.tsx). This is a single line addition.
- `ScrollArea` is a drop-in replacement for `overflow-y-auto` containers.
- Both components will be installed into `src/components/ui/` via `npx shadcn add tooltip scroll-area`.
- Dependencies (`@radix-ui/react-tooltip`, `@radix-ui/react-scroll-area`) are sub-packages of the already-installed `radix-ui`.

---

## R3: Spacing Scale Strategy

### Decision
Adopt a consistent spacing vocabulary using Tailwind v4's default 4px-based scale. Define component-level spacing patterns to follow:

| Context | Gap | Padding | Rationale |
|---------|-----|---------|-----------|
| Between messages | `gap-6` (24px) | — | Comfortable reading rhythm |
| Inside message bubble | — | `px-4 py-3` (16px / 12px) | Readable without wasting space |
| Input area vertical spacing | `space-y-3` (12px) | `px-4 py-4` (16px) | Enough room between controls |
| Sidebar items | — | `px-3 py-2.5` (12px / 10px) | Comfortable click targets |
| Sidebar header | — | `px-4 py-3` (16px / 12px) | Balanced with content |
| Image thumbnails gap | `gap-2` (8px) | — | Tight grouping |
| Feature cards gap | `gap-4` (16px) | `p-5` (20px) | Card-like feel |

### Rationale
The current code has inconsistent spacing — some areas use `py-2` (8px), others `py-3` (12px), and gaps range from `gap-1` (4px) to `gap-4` (16px) without clear reasoning. A defined vocabulary creates visual consistency.

### Key Rules
1. **Eliminate hardcoded pixel font sizes**: Replace `text-[10px]` and `text-[8px]` with `text-xs` (12px) minimum.
2. **Eliminate arbitrary max-width**: Replace `max-w-[75%]` with a responsive approach or standard Tailwind class.
3. **Use `cn()` utility everywhere**: Replace raw template string concatenation with `cn()` from `@/lib/utils` for conditional class composition.

---

## R4: tw-animate-css Animation Strategy

### Decision
Use `tw-animate-css` entrance animations for sidebar content transitions and hover state enhancements. Keep animations subtle and short (150-300ms).

### Available Animations to Use
| Animation | Use Case | Classes |
|-----------|----------|---------|
| Fade in sidebar content | When sidebar expands | `animate-in fade-in duration-200` |
| Slide in sidebar text | Sidebar labels appear | `animate-in fade-in slide-in-from-left-2 duration-200` |
| Delete button reveal | Hover over conversation | `transition-opacity duration-150` (keep existing) |

### Rationale
Animations should be subtle — the goal is "polished", not "flashy". The 200ms duration is fast enough to feel responsive but slow enough to be perceivable. Entrance animations on sidebar content create a professional feel without impacting usability.

---

## R5: ErrorDisplay Color Token Migration

### Decision
Replace hardcoded Tailwind color classes in `ErrorDisplay.tsx` with the existing `--destructive` CSS custom property via Tailwind's `bg-destructive/10`, `border-destructive/50`, `text-destructive` utilities.

### Rationale
The current code at `ErrorDisplay.tsx` uses `border-red-200 bg-red-50 text-red-800` and `dark:border-red-800 dark:bg-red-950 dark:text-red-200`. These bypass the design token system. Using the `destructive` token means:
1. Colors automatically adapt to light/dark mode via CSS variables.
2. No need for explicit `dark:` prefixes.
3. If the destructive color is ever changed, ErrorDisplay updates automatically.

### Before → After
| Current | Replacement |
|---------|-------------|
| `border-red-200 bg-red-50 text-red-800` | `border-destructive/20 bg-destructive/5 text-destructive` |
| `dark:border-red-800 dark:bg-red-950 dark:text-red-200` | (not needed — design tokens handle dark mode) |
| `text-red-600 dark:text-red-400` | `text-destructive/80` |
