<script lang="ts">
  import type { HotkeyConfig, HotkeyBinding, ProgramConfig } from '$lib/types';
  import HotkeyRecorder from './HotkeyRecorder.svelte';
  import FileBrowser from './FileBrowser.svelte';
  import { checkConflict, checkSystemConflict, validateProgramPath } from '$lib/commands';

  interface Props {
    open: boolean;
    hotkey: HotkeyConfig | null;
    onSave: (hotkey: Omit<HotkeyConfig, 'id' | 'createdAt' | 'updatedAt'>) => void;
    onClose: () => void;
  }

  let { open, hotkey, onSave, onClose }: Props = $props();

  // Form state
  let name = $state('');
  let hotkeyBinding = $state<HotkeyBinding>({ modifiers: [], key: '' });
  let programPath = $state('');
  let programArgs = $state('');
  let workingDir = $state('');
  let hidden = $state(false);
  let enabled = $state(true);

  // Validation state
  let errors = $state<Record<string, string>>({});
  let saving = $state(false);

  // Derived state
  const isEdit = $derived(hotkey !== null);
  const title = $derived(isEdit ? 'Edit Hotkey' : 'Add Hotkey');

  // Reset form when dialog opens/closes or hotkey changes
  $effect(() => {
    if (open) {
      if (hotkey) {
        name = hotkey.name;
        hotkeyBinding = { ...hotkey.hotkey };
        programPath = hotkey.program.path;
        programArgs = hotkey.program.arguments.join(' ');
        workingDir = hotkey.program.workingDirectory || '';
        hidden = hotkey.program.hidden;
        enabled = hotkey.enabled;
      } else {
        name = '';
        hotkeyBinding = { modifiers: [], key: '' };
        programPath = '';
        programArgs = '';
        workingDir = '';
        hidden = false;
        enabled = true;
      }
      errors = {};
    }
  });

  async function validate(): Promise<boolean> {
    const newErrors: Record<string, string> = {};

    // Validate name
    if (!name.trim()) {
      newErrors.name = 'Name is required';
    }

    // Validate hotkey
    if (!hotkeyBinding.key) {
      newErrors.hotkey = 'Hotkey is required';
    } else {
      // Check for conflicts
      try {
        const hasConflict = await checkConflict(hotkeyBinding);
        if (hasConflict && !isEdit) {
          newErrors.hotkey = 'This hotkey is already in use';
        }

        const hasSystemConflict = await checkSystemConflict(hotkeyBinding);
        if (hasSystemConflict) {
          newErrors.hotkey = 'This hotkey conflicts with a system shortcut';
        }
      } catch (e) {
        console.error('Failed to check conflicts:', e);
      }
    }

    // Validate program path
    if (!programPath.trim()) {
      newErrors.program = 'Program path is required';
    } else {
      try {
        const isValid = await validateProgramPath(programPath);
        if (!isValid) {
          newErrors.program = 'Program not found or not executable';
        }
      } catch (e) {
        console.error('Failed to validate path:', e);
      }
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  async function handleSubmit() {
    saving = true;
    try {
      const isValid = await validate();
      if (!isValid) {
        return;
      }

      const program: ProgramConfig = {
        path: programPath,
        arguments: programArgs.trim() ? programArgs.split(' ').filter((a) => a) : [],
        workingDirectory: workingDir || undefined,
        hidden,
      };

      onSave({
        name: name.trim(),
        hotkey: hotkeyBinding,
        program,
        enabled,
      });
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-40 bg-black/50 transition-opacity"
    onclick={onClose}
    role="presentation"
  ></div>

  <!-- Dialog -->
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <div
      class="w-full max-w-lg animate-slide-up rounded-lg bg-white shadow-xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b px-6 py-4">
        <h2 id="dialog-title" class="text-lg font-semibold text-gray-900">{title}</h2>
        <button
          onclick={onClose}
          class="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
          aria-label="Close dialog"
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
      </div>

      <!-- Content -->
      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
        class="px-6 py-4"
      >
        <div class="space-y-4">
          <!-- Name -->
          <div>
            <label for="name" class="block text-sm font-medium text-gray-700">Name</label>
            <input
              type="text"
              id="name"
              bind:value={name}
              class="mt-1 block w-full rounded-md border px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500
                {errors.name ? 'border-red-300' : 'border-gray-300'}"
              placeholder="e.g., Launch VS Code"
            />
            {#if errors.name}
              <p class="mt-1 text-sm text-red-600">{errors.name}</p>
            {/if}
          </div>

          <!-- Hotkey -->
          <div>
            <span id="hotkey-label" class="block text-sm font-medium text-gray-700">Hotkey</span>
            <div class="mt-1">
              <HotkeyRecorder
                value={hotkeyBinding}
                onCapture={(hk) => (hotkeyBinding = hk)}
                error={errors.hotkey}
              />
            </div>
          </div>

          <!-- Program Path -->
          <div>
            <FileBrowser
              label="Program"
              value={programPath}
              onChange={(path) => (programPath = path)}
              placeholder="Select an executable..."
              error={errors.program}
            />
          </div>

          <!-- Arguments -->
          <div>
            <label for="args" class="block text-sm font-medium text-gray-700">
              Arguments <span class="text-gray-400">(optional)</span>
            </label>
            <input
              type="text"
              id="args"
              bind:value={programArgs}
              class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              placeholder="e.g., --new-window"
            />
          </div>

          <!-- Working Directory -->
          <div>
            <FileBrowser
              label="Working Directory"
              value={workingDir}
              onChange={(path) => (workingDir = path)}
              placeholder="Use program's directory"
              directory={true}
            />
          </div>

          <!-- Options -->
          <div class="flex items-center gap-6">
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={hidden}
                class="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
              />
              <span class="ml-2 text-sm text-gray-700">Run hidden (no window)</span>
            </label>

            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={enabled}
                class="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
              />
              <span class="ml-2 text-sm text-gray-700">Enabled</span>
            </label>
          </div>
        </div>

        <!-- Footer -->
        <div class="mt-6 flex justify-end gap-3">
          <button
            type="button"
            onclick={onClose}
            class="rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={saving}
            class="inline-flex items-center rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {#if saving}
              <svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24">
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                  fill="none"
                />
                <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
            {/if}
            {isEdit ? 'Save Changes' : 'Add Hotkey'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
