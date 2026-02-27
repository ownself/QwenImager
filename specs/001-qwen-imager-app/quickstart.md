# Quickstart: Qwen AI Image Generation Desktop App

**Feature Branch**: `001-qwen-imager-app`  
**Date**: 2026-02-27

## Prerequisites

### System Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable (latest) | Tauri backend |
| Node.js | 20+ | Frontend toolchain |
| pnpm | 9+ | Package manager (recommended for speed) |

### Platform-specific Dependencies

**Windows**:
- WebView2 (pre-installed on Windows 10/11)
- Visual Studio Build Tools with C++ workload

**macOS**:
- Xcode Command Line Tools (`xcode-select --install`)

**Linux**:
- WebKitGTK 4.1 and dependencies:
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
  # Fedora
  sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
    libappindicator-gtk3-devel librsvg2-devel
  ```

## Setup

### 1. Initialize Tauri Project

```bash
# From repository root
pnpm create tauri-app . --template react-ts --manager pnpm
```

This scaffolds:
- `src-tauri/` - Rust backend with `Cargo.toml` and `tauri.conf.json`
- `src/` - React + TypeScript frontend
- `package.json` - Frontend dependencies
- `vite.config.ts` - Vite bundler config

### 2. Install Frontend Dependencies

```bash
pnpm install

# UI Components
pnpm add tailwindcss @tailwindcss/vite
npx shadcn@latest init

# State Management
pnpm add zustand

# Drag & Drop
pnpm add @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities

# Virtual Scrolling
pnpm add @tanstack/react-virtual

# Dev Dependencies
pnpm add -D vitest @testing-library/react @testing-library/jest-dom
```

### 3. Install Rust Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-store = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-clipboard-manager = "2"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
dirs = "5"
```

### 4. Configure Tauri Plugins

In `src-tauri/src/lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // Register commands here
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5. API Configuration

Create `~/.qwenimage/setting.json`:

```json
{
    "providers": {
        "qwen": {
            "apiKey": "sk-your-api-key-here",
            "models": {
                "qwen-image-max": {
                    "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis"
                },
                "qwen-image-edit-max": {
                    "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation"
                },
                "qwen-mt-image": {
                    "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis"
                }
            }
        }
    }
}
```

## Development

### Run in Development Mode

```bash
pnpm tauri dev
```

This starts:
- Vite dev server with HMR (frontend)
- Cargo build + run (Rust backend)
- Opens the app window

### Run Tests

```bash
# Frontend tests
pnpm test

# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml
```

### Build for Production

```bash
pnpm tauri build
```

Output location:
- Windows: `src-tauri/target/release/bundle/msi/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/deb/` or `appimage/`

## Key Architecture Decisions

| Decision | Choice | Reference |
|----------|--------|-----------|
| Frontend | React + Vite + TypeScript | [research.md](./research.md) §1 |
| Storage | SQLite (rusqlite) | [research.md](./research.md) §2 |
| HTTP Client | reqwest + rustls-tls | [research.md](./research.md) §4 |
| State Management | Zustand | [research.md](./research.md) §1 |
| UI Components | Shadcn/ui + Tailwind | [research.md](./research.md) §1 |
| DnD | @dnd-kit | [research.md](./research.md) §1 |
| Error Handling | thiserror + structured AppError | [research.md](./research.md) §5 |

## File References

- [Specification](./spec.md) - Feature requirements
- [Research](./research.md) - Technology decisions
- [Data Model](./data-model.md) - Entity model and schema
- [Tauri Commands](./contracts/tauri-commands.md) - IPC interface contracts
- [UI Contracts](./contracts/ui-contracts.md) - Component specifications
