# Global Hotkey

A cross-platform desktop application for launching programs and executing AI-assisted workflows via configurable global keyboard shortcuts. Built with Tauri 2, Svelte 5, and Rust.

## Features

### Core Features

- **Global Hotkeys**: Register system-wide keyboard shortcuts that work from any application
- **Program Launcher**: Launch any executable with custom arguments and working directory
- **PATH Support**: Enter program names directly (e.g., `git`, `code`) without full paths
- **Hidden Mode**: Launch CLI applications without visible terminal windows
- **System Tray**: Runs quietly in the background with quick access via tray menu
- **Import/Export**: Backup and restore your hotkey configurations
- **Cross-Platform**: Supports Windows 10/11 and macOS 10.15+
- **Auto-Start**: Optionally start with your system

### AI Integration

- **AI-Powered Hotkeys**: Trigger AI workflows with a single keystroke
- **Voice Input**: Record audio and send to AI for transcription or processing
- **Clipboard Processing**: Send clipboard content to AI with custom instructions
- **Gemini Support**: Integrated with Google Gemini API (gemini-2.5-flash-lite default)
- **Custom Roles**: Create reusable AI roles with custom system prompts
- **Built-in Roles**:
  - DE Transcribe - Transcribe German audio
  - DE→EN Translate - Translate German to English
  - Beautify Text - Improve formatting and clarity
  - Format as AI Response - Structure as professional response

### Post-Actions

Execute automated workflows after a hotkey trigger:

- **Paste Clipboard**: Simulate Ctrl+V (or Cmd+V on macOS)
- **Simulate Keystroke**: Send custom key combinations with modifiers
- **Delay**: Wait a specified time before the next action
- **Trigger Modes**: OnExit (after process completes) or AfterDelay

## Installation

### Windows

Download the latest `.msi` installer from the [Releases](https://github.com/mschnecke/global-hotkey/releases) page.

Or install via Chocolatey:

```bash
choco install global-hotkey
```

### macOS

Download the latest `.pkg` installer from the [Releases](https://github.com/mschnecke/global-hotkey/releases) page (Apple Silicon only).

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
   - Choose action type: **Launch Program** or **Call AI**
   - Configure action-specific settings
   - Optionally configure **Post-Actions** for automation workflows
4. Click **Save** - the hotkey is now active!

### AI Workflows

To use AI features:

1. Go to the **AI Settings** tab
2. Add your Gemini API key and test the connection
3. Create a hotkey with **Call AI** action type
4. Select an AI role and input source (clipboard or audio recording)
5. Optionally add post-actions (e.g., paste the AI response)

**Example: Voice-to-Clipboard**

- Press hotkey → Start recording
- Press hotkey again → Stop recording → AI transcribes → Result saved to clipboard

### Configuration Storage

Configurations are stored in two locations:

- **Settings**: `~/.global-hotkey-settings.json` (fixed location)
- **Hotkeys & AI**: `~/.global-hotkey/config.json` (configurable in General Settings)

## Permissions

### macOS

Global Hotkey requires **Accessibility** permissions to capture keyboard shortcuts:

1. Open **System Settings** > **Privacy & Security** > **Accessibility**
2. Click the lock icon to make changes
3. Enable **Global Hotkey**

For AI audio features, **Microphone** access is also required.

### Windows

No special permissions required for normal operation.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- Platform-specific dependencies:
  - **Windows**: Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools, Opus (`brew install opus`)

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
│   │   ├── ConfirmDialog.svelte
│   │   ├── PostActionEditor.svelte
│   │   ├── AiSettings.svelte
│   │   ├── GeneralSettings.svelte
│   │   └── RoleEditor.svelte
│   ├── lib/                  # Utilities & types
│   └── stores/               # Svelte stores
├── src-tauri/                # Rust backend
│   └── src/
│       ├── ai/               # AI provider integration
│       ├── audio/            # Audio recording & encoding
│       ├── config/           # Configuration management
│       ├── hotkey/           # Global hotkey handling
│       ├── process/          # Process spawning
│       ├── postaction/       # Post-action execution
│       └── tray.rs           # System tray
├── packages/                 # Distribution packages
│   ├── chocolatey/           # Windows package
│   └── macos/                # macOS package
└── docs/                     # Documentation
```

## Documentation

- [Product Requirements Document](docs/PRD.md)
- [Post-Actions Feature Specification](docs/PRD-post-actions.md)
- [AI Module Specification](docs/PRD-ai-module.md)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
