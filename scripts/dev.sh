#!/bin/bash
# Global Hotkey Development Server Script
# Cleans up existing processes and starts the dev server

set -e

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Starting Global Hotkey development server..."

# Port used by Vite dev server
DEV_PORT=1420

# Detect OS
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows (Git Bash / MINGW / Cygwin)
    echo "Detected Windows environment"

    # Kill any process using port 1420
    PID=$(netstat -ano 2>/dev/null | grep ":${DEV_PORT}" | grep "LISTENING" | awk '{print $5}' | head -1)
    if [ -n "$PID" ] && [ "$PID" != "0" ]; then
        echo "Killing process on port ${DEV_PORT} (PID: $PID)"
        taskkill //F //PID "$PID" 2>/dev/null || true
    fi

    # Kill any existing global-hotkey.exe processes
    taskkill //F //IM "global-hotkey.exe" 2>/dev/null || true

else
    # macOS / Linux
    echo "Detected Unix environment"

    # Kill any process using port 1420
    PID=$(lsof -ti:${DEV_PORT} 2>/dev/null || true)
    if [ -n "$PID" ]; then
        echo "Killing process on port ${DEV_PORT} (PID: $PID)"
        kill -9 $PID 2>/dev/null || true
    fi

    # Kill any existing global-hotkey processes
    pkill -f "global-hotkey" 2>/dev/null || true
    pkill -f "target/debug/global-hotkey" 2>/dev/null || true
    pkill -f "target/release/global-hotkey" 2>/dev/null || true
fi

# Wait for port to be freed
sleep 1

# Verify port is free
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    if netstat -ano 2>/dev/null | grep ":${DEV_PORT}" | grep -q "LISTENING"; then
        echo "Warning: Port ${DEV_PORT} is still in use"
    fi
else
    if lsof -ti:${DEV_PORT} 2>/dev/null; then
        echo "Warning: Port ${DEV_PORT} is still in use"
    fi
fi

echo "Starting Tauri development server..."
npm run tauri dev
