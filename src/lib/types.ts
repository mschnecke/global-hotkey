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
  postActions?: PostActionsConfig;
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

// Post-Action Types
export type PostActionTrigger = { type: 'onExit' } | { type: 'afterDelay'; delayMs: number };

export interface Keystroke {
  modifiers: string[];
  key: string;
}

export type PostActionType =
  | { type: 'pasteClipboard' }
  | { type: 'simulateKeystroke'; keystroke: Keystroke }
  | { type: 'delay'; delayMs: number };

export interface PostAction {
  id: string;
  actionType: PostActionType;
  enabled: boolean;
}

export interface PostActionsConfig {
  enabled: boolean;
  trigger: PostActionTrigger;
  actions: PostAction[];
}
