# Global Hotkey - Agent System Instructions

## System Overview

This document serves as the central hub for the Global Hotkey project's agent system. It provides routing guidance to determine which specialized agent should handle specific tasks during development.

**How it works:** When a development task is requested, consult the routing table below to identify the appropriate agent. Each agent contains detailed instructions, behaviors, and examples for its domain. Read the full agent file before executing tasks in that domain.

**Progressive Disclosure:** Start with this file for routing decisions, then dive into specific agent files for detailed implementation guidance.

---

## Project Structure

```
global-hotkey/
├── .claude/
│   ├── INSTRUCTIONS.md          # This file - agent routing hub
│   ├── settings.local.json      # Claude Code settings
│   └── agents/                  # Specialized agent definitions
│       ├── tauri-scaffolding.md
│       ├── rust-backend.md
│       ├── svelte-frontend.md
│       ├── system-tray.md
│       ├── ci-cd-setup.md
│       └── platform-integration.md
├── docs/
│   ├── PRD.md                   # Product Requirements Document
│   └── implementation-plan.md   # Detailed implementation phases
├── src/                         # Svelte frontend (to be created)
├── src-tauri/                   # Rust backend (to be created)
├── .github/workflows/           # CI/CD pipelines (to be created)
└── packages/                    # Distribution packages (to be created)
```

---

## Available Agents

### 1. Tauri Scaffolding (`agents/tauri-scaffolding.md`)

**Purpose:** Initialize and configure Tauri 2.x projects with Svelte, TypeScript, and Tailwind CSS.

**When to use:**

- Setting up a new Tauri desktop application from scratch
- Adding Tauri to an existing project
- Configuring development tooling (ESLint, Prettier, Husky)
- Setting up Tailwind CSS with Svelte
- Creating initial project folder structure

**Output:** Project root with `package.json`, `src/`, `src-tauri/`, and config files

**To use this agent:** Read `.claude/agents/tauri-scaffolding.md`

---

### 2. Rust Backend (`agents/rust-backend.md`)

**Purpose:** Implement Rust backend modules for Tauri including configuration management, hotkey handling, and process spawning.

**When to use:**

- Implementing Tauri commands
- Creating data structures for configuration
- Writing file system operations
- Implementing cross-platform system integrations
- Adding Rust dependencies

**Output:** Rust modules in `src-tauri/src/`

**To use this agent:** Read `.claude/agents/rust-backend.md`

---

### 3. Svelte Frontend (`agents/svelte-frontend.md`)

**Purpose:** Build Svelte 5 components with TypeScript and Tailwind CSS for the settings UI.

**When to use:**

- Creating new Svelte components
- Implementing forms, dialogs, and interactive UI
- Setting up Svelte stores for state management
- Writing TypeScript interfaces
- Creating Tauri command wrapper functions

**Output:** Svelte components in `src/components/`, stores in `src/stores/`

**To use this agent:** Read `.claude/agents/svelte-frontend.md`

---

### 4. System Tray (`agents/system-tray.md`)

**Purpose:** Configure and implement system tray functionality including icons, menus, and background operation.

**When to use:**

- Setting up system tray icon
- Creating and updating tray context menus
- Implementing menu item click handlers
- Configuring app to run in background
- Adding "Start with System" functionality

**Output:** Tray implementation in `src-tauri/src/tray.rs`

**To use this agent:** Read `.claude/agents/system-tray.md`

---

### 5. CI/CD Setup (`agents/ci-cd-setup.md`)

**Purpose:** Configure GitHub Actions workflows, Git hooks, and deployment pipelines.

**When to use:**

- Setting up GitHub Actions CI workflow
- Creating release automation
- Configuring Husky and lint-staged
- Setting up Chocolatey package (Windows)
- Creating Homebrew cask (macOS)

**Output:** Workflows in `.github/workflows/`, packages in `packages/`

**To use this agent:** Read `.claude/agents/ci-cd-setup.md`

---

### 6. Platform Integration (`agents/platform-integration.md`)

**Purpose:** Implement Windows and macOS specific functionality.

**When to use:**

- Implementing Windows-specific features (Registry, Windows API)
- Implementing macOS-specific features (Accessibility, LaunchAgents)
- Handling platform permission requirements
- Creating platform-specific auto-start
- Hidden/background process launching

**Output:** Platform modules in `src-tauri/src/platform/`

**To use this agent:** Read `.claude/agents/platform-integration.md`

---

## Routing Table

| User Request                      | Agent                |
| --------------------------------- | -------------------- |
| "Set up the project"              | tauri-scaffolding    |
| "Initialize Tauri"                | tauri-scaffolding    |
| "Add Tailwind"                    | tauri-scaffolding    |
| "Configure ESLint/Prettier"       | tauri-scaffolding    |
| "Set up Husky"                    | tauri-scaffolding    |
| "Implement config loading/saving" | rust-backend         |
| "Create Tauri command"            | rust-backend         |
| "Add Rust dependency"             | rust-backend         |
| "Implement hotkey registration"   | rust-backend         |
| "Launch programs from backend"    | rust-backend         |
| "Create a component"              | svelte-frontend      |
| "Build the settings UI"           | svelte-frontend      |
| "Add a Svelte store"              | svelte-frontend      |
| "Implement hotkey recorder"       | svelte-frontend      |
| "Create dialog/modal"             | svelte-frontend      |
| "Set up system tray"              | system-tray          |
| "Add tray menu item"              | system-tray          |
| "Run in background"               | system-tray          |
| "Minimize to tray"                | system-tray          |
| "Set up CI/CD"                    | ci-cd-setup          |
| "Create GitHub Action"            | ci-cd-setup          |
| "Configure release workflow"      | ci-cd-setup          |
| "Set up Chocolatey"               | ci-cd-setup          |
| "Create Homebrew cask"            | ci-cd-setup          |
| "Windows Registry"                | platform-integration |
| "macOS permissions"               | platform-integration |
| "Auto-start on boot"              | platform-integration |
| "Hidden window launch"            | platform-integration |
| "Platform-specific code"          | platform-integration |

---

## Workflow Patterns

### Sequential Workflow (Recommended for Initial Setup)

```
1. tauri-scaffolding  → Project foundation
2. rust-backend       → Config manager first
3. svelte-frontend    → Basic UI shell
4. rust-backend       → Hotkey + process modules
5. svelte-frontend    → Complete UI components
6. system-tray        → Background operation
7. platform-integration → OS-specific features
8. ci-cd-setup        → Automation
```

### Parallel Workflow (After Foundation)

Once scaffolding is complete, these can often work in parallel:

- `rust-backend` + `svelte-frontend` (backend and frontend development)
- `system-tray` + `platform-integration` (system integrations)

### Feature Workflow

For adding a new feature end-to-end:

```
rust-backend → svelte-frontend → (system-tray if tray-related)
```

---

## File Naming Conventions

| Type              | Convention          | Example                 |
| ----------------- | ------------------- | ----------------------- |
| Rust modules      | snake_case          | `config_manager.rs`     |
| Svelte components | PascalCase          | `HotkeyList.svelte`     |
| TypeScript        | camelCase files     | `commands.ts`           |
| Stores            | camelCase           | `hotkeys.ts`            |
| Config files      | lowercase with dots | `tailwind.config.js`    |
| GitHub workflows  | kebab-case          | `ci.yml`, `release.yml` |

---

## Git Commit Protocol

**When to commit:**

- After completing a logical unit of work
- After each implementation phase
- Before switching between agents

**Commit message format:**

```
type(scope): description

- Detail 1
- Detail 2
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Scopes:** `frontend`, `backend`, `tray`, `ci`, `platform`

---

## Adding New Agents

To add a new agent to this system:

1. Create agent file in `.claude/agents/` following the template:
   - Header & Description
   - Purpose
   - When to Use
   - Core Behaviors (numbered)
   - Output Format
   - Output Location
   - Examples

2. Add entry to "Available Agents" section above

3. Add routing entries to the "Routing Table"

4. Update "Workflow Patterns" if the agent affects sequencing

---

## Quick Start

1. **Read the PRD:** `docs/PRD.md` for full requirements
2. **Review the plan:** `docs/implementation-plan.md` for phases
3. **Start with scaffolding:** Use `tauri-scaffolding` agent first
4. **Follow the sequential workflow** for initial development
5. **Consult routing table** when unsure which agent to use
