<script lang="ts">
  import type { AiRole, OutputFormat } from '$lib/types';

  interface Props {
    open: boolean;
    role: AiRole | null;
    onSave: (role: AiRole) => void;
    onClose: () => void;
  }

  let { open, role, onSave, onClose }: Props = $props();

  // Form state
  let name = $state('');
  let systemPrompt = $state('');
  let outputFormat = $state<OutputFormat>('plain');

  // Validation state
  let errors = $state<Record<string, string>>({});

  // Derived state
  const isEdit = $derived(role !== null);
  const title = $derived(isEdit ? 'Edit Role' : 'Add Custom Role');

  // Reset form when dialog opens/closes or role changes
  $effect(() => {
    if (open) {
      if (role) {
        name = role.name;
        systemPrompt = role.systemPrompt;
        outputFormat = role.outputFormat;
      } else {
        name = '';
        systemPrompt = '';
        outputFormat = 'plain';
      }
      errors = {};
    }
  });

  function validate(): boolean {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (!systemPrompt.trim()) {
      newErrors.systemPrompt = 'System prompt is required';
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  function handleSubmit() {
    if (!validate()) {
      return;
    }

    const savedRole: AiRole = {
      id: role?.id || crypto.randomUUID(),
      name: name.trim(),
      systemPrompt: systemPrompt.trim(),
      outputFormat,
      isBuiltin: false,
    };

    onSave(savedRole);
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
      class="w-full max-w-lg max-h-[90vh] flex flex-col animate-slide-up rounded-lg bg-white shadow-xl"
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
        class="px-6 py-4 overflow-y-auto flex-1"
      >
        <div class="space-y-4">
          <!-- Name -->
          <div>
            <label for="role-name" class="block text-sm font-medium text-gray-700">Name</label>
            <input
              type="text"
              id="role-name"
              bind:value={name}
              class="mt-1 block w-full rounded-md border px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500
                {errors.name ? 'border-red-300' : 'border-gray-300'}"
              placeholder="e.g., Summarize Text"
            />
            {#if errors.name}
              <p class="mt-1 text-sm text-red-600">{errors.name}</p>
            {/if}
          </div>

          <!-- System Prompt -->
          <div>
            <label for="system-prompt" class="block text-sm font-medium text-gray-700">
              System Prompt
            </label>
            <textarea
              id="system-prompt"
              bind:value={systemPrompt}
              rows={6}
              class="mt-1 block w-full rounded-md border px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500
                {errors.systemPrompt ? 'border-red-300' : 'border-gray-300'}"
              placeholder="Instructions for the AI..."
            ></textarea>
            {#if errors.systemPrompt}
              <p class="mt-1 text-sm text-red-600">{errors.systemPrompt}</p>
            {/if}
            <p class="mt-1 text-xs text-gray-500">
              This prompt tells the AI how to process the input text.
            </p>
          </div>

          <!-- Output Format -->
          <div>
            <label for="output-format" class="block text-sm font-medium text-gray-700">
              Output Format
            </label>
            <select
              id="output-format"
              bind:value={outputFormat}
              class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            >
              <option value="plain">Plain Text</option>
              <option value="markdown">Markdown</option>
              <option value="json">JSON</option>
            </select>
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
            class="inline-flex items-center rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
          >
            {isEdit ? 'Save Changes' : 'Add Role'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
