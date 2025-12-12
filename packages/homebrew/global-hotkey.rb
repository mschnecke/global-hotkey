# Homebrew Cask for Global Hotkey
# This file should be placed in a homebrew-global-hotkey tap repository
# at Casks/global-hotkey.rb

cask "global-hotkey" do
  version "0.1.0"

  on_intel do
    sha256 "REPLACE_WITH_INTEL_DMG_SHA256"
    url "https://github.com/mschnecke/global-hotkey/releases/download/v#{version}/Global.Hotkey_#{version}_x64.dmg"
  end

  on_arm do
    sha256 "REPLACE_WITH_ARM_DMG_SHA256"
    url "https://github.com/mschnecke/global-hotkey/releases/download/v#{version}/Global.Hotkey_#{version}_aarch64.dmg"
  end

  name "Global Hotkey"
  desc "Launch programs with global keyboard shortcuts"
  homepage "https://github.com/mschnecke/global-hotkey"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :catalina"

  app "Global Hotkey.app"

  postflight do
    # Request accessibility permissions
    system_command "/usr/bin/osascript",
                   args: ["-e", 'tell application "System Events" to keystroke ""'],
                   sudo: false
  end

  zap trash: [
    "~/Library/Application Support/com.globalhotkey.app",
    "~/Library/Caches/com.globalhotkey.app",
    "~/Library/LaunchAgents/com.globalhotkey.app.plist",
    "~/Library/Preferences/com.globalhotkey.app.plist",
  ]
end
