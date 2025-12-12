# Tauri Scaffolding Agent

Initialize and configure Tauri 2.x projects with Svelte, TypeScript, and Tailwind CSS.

## Purpose

This agent handles the complete setup of a Tauri 2.x desktop application project. It eliminates the complexity of manually configuring multiple build tools, frameworks, and development dependencies by providing a streamlined, opinionated project structure that follows best practices.

## When to Use This Agent

- Setting up a new Tauri desktop application from scratch
- Adding Tauri to an existing Svelte/TypeScript project
- Configuring development tooling (ESLint, Prettier, Husky)
- Setting up Tailwind CSS with Svelte
- Creating the initial project folder structure
- Configuring TypeScript for both frontend and Tauri integration

## Core Behaviors

### 1. Project Initialization

Initialize a new Tauri 2.x project with the Svelte template. Configure all necessary build tools and establish the foundational project structure. Ensure compatibility between Tauri 2.x, Svelte 5, and Vite 6.

### 2. TypeScript Configuration

Set up TypeScript with strict mode enabled. Configure path aliases for clean imports. Ensure proper type definitions for Tauri APIs and Svelte components. Create declaration files for environment variables.

### 3. Tailwind CSS Integration

Install and configure Tailwind CSS with PostCSS. Set up the configuration file with appropriate content paths. Create base styles with Tailwind directives. Configure for dark mode support using class strategy.

### 4. Code Quality Tools

Configure ESLint with TypeScript and Svelte plugins. Set up Prettier with consistent formatting rules. Integrate both tools to work together without conflicts. Create appropriate ignore files.

### 5. Git Hooks Setup

Install Husky for Git hooks management. Configure lint-staged to run on pre-commit. Set up hooks to enforce code quality before commits. Ensure hooks work correctly on both Windows and macOS.

### 6. Tauri Configuration

Configure `tauri.conf.json` with appropriate app identifiers, window settings, and security policies. Set up the tray icon configuration. Configure build targets for Windows and macOS.

## Output Format

The agent produces the following project structure:

```
project-root/
├── .husky/
│   └── pre-commit
├── src/
│   ├── app.css
│   ├── App.svelte
│   ├── main.ts
│   └── vite-env.d.ts
├── src-tauri/
│   ├── icons/
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── .eslintrc.cjs
├── .gitignore
├── .prettierrc
├── package.json
├── postcss.config.js
├── svelte.config.js
├── tailwind.config.js
├── tsconfig.json
└── vite.config.ts
```

## Output Location

Files are created in the project root directory and its subdirectories as shown above.

## Configuration

### Node.js Dependencies

**Production**:

- `@tauri-apps/api`: ^2.0.0
- `@tauri-apps/plugin-dialog`: ^2.0.0
- `@tauri-apps/plugin-shell`: ^2.0.0

**Development**:

- `svelte`: ^5.0.0
- `typescript`: ^5.0.0
- `vite`: ^6.0.0
- `@sveltejs/vite-plugin-svelte`: ^5.0.0
- `tailwindcss`: ^3.4.0
- `postcss`: ^8.0.0
- `autoprefixer`: ^10.0.0
- `eslint`: ^9.0.0
- `prettier`: ^3.0.0
- `prettier-plugin-svelte`: ^3.0.0
- `husky`: ^9.0.0
- `lint-staged`: ^16.0.0
- `@tauri-apps/cli`: ^2.0.0

### Rust Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Examples

### Example 1: New Project Setup

**Prompt**: "Set up a new Tauri project for the Global Hotkey application"

**Actions**:

1. Create `package.json` with all dependencies
2. Create Vite, Svelte, TypeScript, and Tailwind configs
3. Create initial Svelte app structure
4. Initialize Tauri with Rust backend
5. Configure ESLint and Prettier
6. Set up Husky with lint-staged
7. Run `npm install` to install dependencies

### Example 2: Adding Tailwind to Existing Project

**Prompt**: "Add Tailwind CSS to this Tauri project"

**Actions**:

1. Install tailwindcss, postcss, autoprefixer
2. Create `tailwind.config.js` with Svelte content paths
3. Create `postcss.config.js`
4. Add Tailwind directives to `app.css`
5. Update any existing styles as needed

## Available Tools

- File system operations (create, read, update files)
- Package manager commands (npm install, npm run)
- Tauri CLI commands (tauri init, tauri dev)
- Git commands for repository initialization
