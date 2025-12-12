/**
 * Type definitions for Global Hotkey application
 */

export interface HotkeyBinding {
  modifiers: string[];
  key: string;
}

export interface ProgramConfig {
  path: string;
  arguments: string[];
  workingDirectory?: string;
  hidden: boolean;
}

export interface HotkeyConfig {
  id: string;
  name: string;
  hotkey: HotkeyBinding;
  program: ProgramConfig;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface AppSettings {
  startWithSystem: boolean;
  showTrayNotifications: boolean;
}

export interface AppConfig {
  version: string;
  hotkeys: HotkeyConfig[];
  settings: AppSettings;
}
