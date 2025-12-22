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

// Main action types for hotkeys
export type HotkeyAction =
  | { type: 'launchProgram'; program: ProgramConfig }
  | { type: 'callAi'; roleId: string; inputSource: AiInputSource; providerId?: string };

export interface HotkeyConfig {
  id: string;
  name: string;
  hotkey: HotkeyBinding;
  action: HotkeyAction;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
  postActions?: PostActionsConfig;
}

/**
 * Application settings stored in fixed location (~/.global-hotkey-settings.json)
 */
export interface AppSettings {
  startWithSystem: boolean;
  showTrayNotifications: boolean;
  /** Custom config location. If undefined, uses default (~/.global-hotkey/) */
  configLocation?: string;
}

/**
 * Main configuration stored in configurable location
 */
export interface AppConfig {
  version: string;
  hotkeys: HotkeyConfig[];
  ai: AiSettings;
}

/**
 * Combined settings and config for full app state
 */
export interface FullConfig {
  settings: AppSettings;
  config: AppConfig;
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

// ============================================================================
// AI Module Types
// ============================================================================

export type AiProviderType = 'gemini' | 'openai' | 'anthropic' | 'ollama';

export interface AiProviderConfig {
  id: string;
  providerType: AiProviderType;
  apiKey: string;
  model?: string;
  baseUrl?: string;
  enabled: boolean;
}

export type OutputFormat = 'plain' | 'markdown' | 'json';

export interface AiRole {
  id: string;
  name: string;
  systemPrompt: string;
  outputFormat: OutputFormat;
  isBuiltin: boolean;
}

export interface AiSettings {
  providers: AiProviderConfig[];
  defaultProviderId?: string;
  roles: AiRole[];
}

export type AudioFormat = 'opus' | 'wav';

export type AiInputSource =
  | { type: 'clipboard' }
  | { type: 'recordAudio'; maxDurationMs: number; format: AudioFormat }
  | { type: 'processOutput' };
