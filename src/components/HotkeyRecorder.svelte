<script lang="ts">
  import type { HotkeyBinding } from '$lib/types';

  interface Props {
    value: HotkeyBinding | null;
    onCapture: (hotkey: HotkeyBinding) => void;
    error?: string;
  }

  let { value, onCapture, error }: Props = $props();

  let recording = $state(false);
  let currentModifiers = $state<string[]>([]);
  let buttonRef: HTMLButtonElement | null = $state(null);

  function formatHotkey(hk: HotkeyBinding | null): string {
    if (!hk) return 'Click to record';
    const parts = [
      ...hk.modifiers.map((m) => m.charAt(0).toUpperCase() + m.slice(1)),
      hk.key.toUpperCase(),
    ];
    return parts.join(' + ');
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();

    // Track current modifiers
    const modifiers: string[] = [];
    if (e.ctrlKey) modifiers.push('Ctrl');
    if (e.altKey) modifiers.push('Alt');
    if (e.shiftKey) modifiers.push('Shift');
    if (e.metaKey) modifiers.push('Meta');

    currentModifiers = modifiers;

    // Check if it's a non-modifier key
    const key = e.key;
    const isModifierOnly = ['Control', 'Alt', 'Shift', 'Meta'].includes(key);

    if (!isModifierOnly && modifiers.length > 0) {
      // Map key names to our format
      let mappedKey = key;
      if (key.length === 1) {
        mappedKey = key.toUpperCase();
      } else if (key.startsWith('Arrow')) {
        mappedKey = key.replace('Arrow', '');
      }

      onCapture({ modifiers, key: mappedKey });
      recording = false;
      currentModifiers = [];
    }
  }

  function handleKeyUp(e: KeyboardEvent) {
    if (!recording) return;

    // Update current modifiers on key up
    const modifiers: string[] = [];
    if (e.ctrlKey) modifiers.push('Ctrl');
    if (e.altKey) modifiers.push('Alt');
    if (e.shiftKey) modifiers.push('Shift');
    if (e.metaKey) modifiers.push('Meta');
    currentModifiers = modifiers;
  }

  function handleBlur() {
    recording = false;
    currentModifiers = [];
  }

  function startRecording() {
    recording = true;
    currentModifiers = [];
    // Ensure the button has focus to receive key events
    buttonRef?.focus();
  }

  function clearHotkey() {
    onCapture({ modifiers: [], key: '' });
  }
</script>

<div class="flex items-center gap-2">
  <button
    type="button"
    bind:this={buttonRef}
    class="flex-1 rounded-md border px-4 py-2 text-left font-mono text-sm transition-all
      {recording
      ? 'border-primary-500 ring-2 ring-primary-500 bg-primary-50'
      : error
        ? 'border-red-300 bg-red-50'
        : 'border-gray-300 bg-white hover:border-gray-400'}"
    onclick={startRecording}
    onkeydown={handleKeyDown}
    onkeyup={handleKeyUp}
    onblur={handleBlur}
  >
    {#if recording}
      <span class="text-primary-600">
        {#if currentModifiers.length > 0}
          {currentModifiers.join(' + ')} + ...
        {:else}
          Press a key combination...
        {/if}
      </span>
    {:else}
      <span class={value?.key ? 'text-gray-900' : 'text-gray-400'}>
        {formatHotkey(value?.key ? value : null)}
      </span>
    {/if}
  </button>

  {#if value?.key}
    <button
      type="button"
      onclick={clearHotkey}
      class="rounded-md p-2 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
      title="Clear hotkey"
    >
      <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M6 18L18 6M6 6l12 12"
        />
      </svg>
    </button>
  {/if}
</div>

{#if error}
  <p class="mt-1 text-sm text-red-600">{error}</p>
{/if}
