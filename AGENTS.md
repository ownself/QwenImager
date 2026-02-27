# QwenImager Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-27

## Active Technologies
- SQLite (rusqlite with bundled feature) (002-image-base64-upload)
- Rust (stable, latest) + TypeScript 5.x + Tauri v2, React 19, reqwest, serde_json, rusqlite, shadcn/ui, zustand (003-config-driven-api)
- SQLite (rusqlite with bundled feature) — 本功能不修改 schema (003-config-driven-api)
- TypeScript 5.x + React 19 + Tailwind CSS v4, shadcn/ui (Tooltip + ScrollArea), tw-animate-css, lucide-react, @dnd-kit (004-ui-layout-polish)
- N/A (no data changes) (004-ui-layout-polish)

- Rust (stable, latest) + TypeScript 5.x + Tauri v2, React 19, Vite, reqwest, rusqlite, shadcn/ui, zustand, @dnd-kit (001-qwen-imager-app)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test; cargo clippy

## Code Style

Rust (stable, latest) + TypeScript 5.x: Follow standard conventions

## Recent Changes
- 004-ui-layout-polish: Added TypeScript 5.x + React 19 + Tailwind CSS v4, shadcn/ui (Tooltip + ScrollArea), tw-animate-css, lucide-react, @dnd-kit
- 003-config-driven-api: Added Rust (stable, latest) + TypeScript 5.x + Tauri v2, React 19, reqwest, serde_json, rusqlite, shadcn/ui, zustand
- 002-image-base64-upload: Added Rust (stable, latest) + TypeScript 5.x + Tauri v2, React 19, Vite, reqwest, rusqlite, shadcn/ui, zustand, @dnd-kit


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
