# Global Hotkey

A cross-platform desktop application for launching programs via configurable global keyboard shortcuts. Built with Tauri 2, Svelte 5, and Rust.

## Features

- **Global Hotkeys**: Register system-wide keyboard shortcuts that work from any application
- **Program Launcher**: Launch any executable with custom arguments and working directory
- **System Tray**: Runs quietly in the background with quick access via tray menu
- **Hidden Mode**: Launch CLI applications without visible terminal windows
- **Import/Export**: Backup and restore your hotkey configurations
- **Cross-Platform**: Supports Windows 10/11 and macOS 10.15+
- **Auto-Start**: Optionally start with your system

## Installation

### Windows

Download the latest `.msi` installer from the [Releases](https://github.com/mschnecke/global-hotkey/releases) page.

Or install via Chocolatey:

```bash
choco install global-hotkey
```

### macOS

Download the latest `.dmg` from the [Releases](https://github.com/mschnecke/global-hotkey/releases) page.

Or install via Homebrew:

```bash
brew tap mschnecke/global-hotkey
brew install --cask global-hotkey
```

## Usage

1. Launch the application - it will appear in your system tray
2. Right-click the tray icon and select **Settings** to open the configuration window
3. Click **Add Hotkey** to create a new shortcut:
   - Enter a name for the hotkey
   - Click the hotkey recorder and press your desired key combination
   - Browse to select the program to launch
   - Optionally set arguments, working directory, or hidden mode
4. Click **Save** - the hotkey is now active!

### Configuration Storage

Configurations are stored at `~/.global-hotkey.json` (in your home directory) on all platforms.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- Platform-specific dependencies:
  - **Windows**: Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools

### Setup

```bash
# Clone the repository
git clone https://github.com/mschnecke/global-hotkey.git
cd global-hotkey

# Install dependencies
npm install

# Start development server
npm run tauri:dev
```

### Build Commands

```bash
# Development
npm run dev           # Start Vite dev server only
npm run tauri:dev     # Start full Tauri app in dev mode

# Production Build
npm run build         # Build frontend only
npm run tauri:build   # Build complete application installer

# Code Quality
npm run check         # TypeScript/Svelte type checking
npm run lint          # Run ESLint
npm run lint:fix      # ESLint with auto-fix
npm run format        # Format with Prettier
npm run format:check  # Check formatting
```

## Tech Stack

| Layer     | Technology                               | Version      |
| --------- | ---------------------------------------- | ------------ |
| Framework | [Tauri](https://tauri.app/)              | 2.x          |
| Frontend  | [Svelte](https://svelte.dev/)            | 5.x          |
| Language  | TypeScript                               | 5.x          |
| Styling   | [Tailwind CSS](https://tailwindcss.com/) | 3.x          |
| Build     | [Vite](https://vitejs.dev/)              | 6.x          |
| Backend   | Rust                                     | 2021 Edition |

## Project Structure

```
global-hotkey/
├── src/                      # Svelte frontend
│   ├── components/           # UI components
│   │   ├── HotkeyList.svelte
│   │   ├── HotkeyDialog.svelte
│   │   ├── HotkeyRecorder.svelte
│   │   ├── FileBrowser.svelte
│   │   └── ConfirmDialog.svelte
│   ├── lib/                  # Utilities & types
│   └── stores/               # Svelte stores
├── src-tauri/                # Rust backend
│   └── src/
│       ├── config/           # Configuration management
│       ├── hotkey/           # Global hotkey handling
│       ├── process/          # Process spawning
│       └── tray.rs           # System tray
├── packages/                 # Distribution packages
│   ├── chocolatey/           # Windows package
│   └── homebrew/             # macOS package
└── docs/                     # Documentation
```

## Documentation

- [Product Requirements Document](docs/PRD.md)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
