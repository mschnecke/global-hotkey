<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    value: string;
    onChange: (path: string) => void;
    label?: string;
    placeholder?: string;
    directory?: boolean;
    error?: string;
  }

  let {
    value,
    onChange,
    label,
    placeholder = 'Select a file...',
    directory = false,
    error,
  }: Props = $props();

  let loading = $state(false);

  async function browseFile() {
    loading = true;
    try {
      const selected = await open({
        multiple: false,
        directory,
        filters: directory
          ? undefined
          : [
              {
                name: 'Executables',
                extensions: ['exe', 'bat', 'cmd', 'ps1', 'com', 'app', '*'],
              },
              {
                name: 'All Files',
                extensions: ['*'],
              },
            ],
      });

      if (selected && typeof selected === 'string') {
        onChange(selected);
      }
    } catch (e) {
      console.error('Failed to open file dialog:', e);
    } finally {
      loading = false;
    }
  }

  function getFilename(path: string): string {
    if (!path) return '';
    const separator = path.includes('\\') ? '\\' : '/';
    const parts = path.split(separator);
    return parts[parts.length - 1] || path;
  }
</script>

<div>
  {#if label}
    <span class="mb-1 block text-sm font-medium text-gray-700">{label}</span>
  {/if}

  <div class="flex items-center gap-2">
    <div
      class="flex-1 truncate rounded-md border px-3 py-2 text-sm
        {error ? 'border-red-300 bg-red-50' : 'border-gray-300 bg-gray-50'}"
      title={value}
    >
      {#if value}
        <span class="text-gray-900">{getFilename(value)}</span>
        <span class="ml-2 text-xs text-gray-500">{value}</span>
      {:else}
        <span class="text-gray-400">{placeholder}</span>
      {/if}
    </div>

    <button
      type="button"
      onclick={browseFile}
      disabled={loading}
      class="inline-flex items-center rounded-md border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
    >
      {#if loading}
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
      {:else}
        <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
          />
        </svg>
      {/if}
      Browse
    </button>

    {#if value}
      <button
        type="button"
        onclick={() => onChange('')}
        class="rounded-md p-2 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
        title="Clear"
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
</div>
