# CLAUDE.md

**CRITICAL: Read `.claude/INSTRUCTIONS.md` immediately.**

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Global Hotkey is a cross-platform application for launching programs via configurable keyboard shortcuts.

## Tech Stack

- **Framework**: Tauri 2.x (Rust backend)
- **Frontend**: Svelte 5 + TypeScript + Tailwind CSS
- **Build**: Vite 6
- **Platforms**: Windows 10/11, macOS 10.15+

## Build Commands

```bash
# Install dependencies
npm install

# Development
npm run dev        # Start Vite dev server
npm run tauri:dev  # Start Tauri app in dev mode

# Build
npm run build        # Build frontend only
npm run tauri:build  # Build full Tauri application

# Code Quality
npm run check       # TypeScript/Svelte type checking
npm run lint        # ESLint
npm run lint:fix    # ESLint with auto-fix
npm run format      # Prettier format
npm run format:check # Prettier check
```

## Project Structure

```
global-hotkey/
├── src/                    # Svelte frontend
│   ├── components/         # UI components
│   ├── lib/               # Utilities, types, commands
│   └── stores/            # Svelte stores
├── src-tauri/             # Rust backend
│   └── src/
│       ├── config/        # Configuration management
│       ├── hotkey/        # Global hotkey handling
│       ├── process/       # Process spawning
│       └── tray.rs        # System tray
├── .husky/                # Git hooks
└── docs/                  # Documentation
```

## Development Status

**Current Status**: Core implementation complete (Phases 1-5)

| Phase | Status |
|-------|--------|
| 1. Project Scaffolding | Complete |
| 2. Rust Backend Core | Complete |
| 3. System Tray Integration | Complete |
| 4. Svelte Frontend | Complete |
| 5. Import/Export & Backup | Complete |
| 6. Platform-Specific Features | In Progress |
| 7. CI/CD & Deployment | Configured |

See `docs/implementation-plan.md` for detailed phase descriptions.
