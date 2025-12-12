<script lang="ts">
  import type { HotkeyConfig } from '$lib/types';

  interface Props {
    hotkeys: HotkeyConfig[];
    onEdit: (hotkey: HotkeyConfig) => void;
    onDelete: (hotkey: HotkeyConfig) => void;
    onToggle: (hotkey: HotkeyConfig) => void;
  }

  let { hotkeys, onEdit, onDelete, onToggle }: Props = $props();

  function formatHotkey(hk: HotkeyConfig['hotkey']): string {
    const parts = [
      ...hk.modifiers.map((m) => m.charAt(0).toUpperCase() + m.slice(1)),
      hk.key.toUpperCase(),
    ];
    return parts.join(' + ');
  }

  function getFilename(path: string): string {
    const separator = path.includes('\\') ? '\\' : '/';
    const parts = path.split(separator);
    return parts[parts.length - 1] || path;
  }
</script>

<div class="overflow-hidden rounded-lg border border-gray-200 bg-white shadow">
  {#if hotkeys.length === 0}
    <div class="px-6 py-12 text-center">
      <svg
        class="mx-auto h-12 w-12 text-gray-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M12 6v6m0 0v6m0-6h6m-6 0H6"
        />
      </svg>
      <h3 class="mt-2 text-sm font-semibold text-gray-900">No hotkeys configured</h3>
      <p class="mt-1 text-sm text-gray-500">Get started by creating a new hotkey.</p>
    </div>
  {:else}
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th
            scope="col"
            class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
          >
            Name
          </th>
          <th
            scope="col"
            class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
          >
            Hotkey
          </th>
          <th
            scope="col"
            class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
          >
            Program
          </th>
          <th
            scope="col"
            class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500"
          >
            Status
          </th>
          <th scope="col" class="relative px-6 py-3">
            <span class="sr-only">Actions</span>
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-200 bg-white">
        {#each hotkeys as hotkey (hotkey.id)}
          <tr class="hover:bg-gray-50 transition-colors">
            <td class="whitespace-nowrap px-6 py-4">
              <div class="text-sm font-medium text-gray-900">{hotkey.name}</div>
            </td>
            <td class="whitespace-nowrap px-6 py-4">
              <code class="rounded bg-gray-100 px-2 py-1 font-mono text-sm text-gray-800">
                {formatHotkey(hotkey.hotkey)}
              </code>
            </td>
            <td class="max-w-xs truncate px-6 py-4">
              <div class="text-sm text-gray-900" title={hotkey.program.path}>
                {getFilename(hotkey.program.path)}
              </div>
              {#if hotkey.program.hidden}
                <span class="text-xs text-gray-500">Hidden</span>
              {/if}
            </td>
            <td class="whitespace-nowrap px-6 py-4">
              <button
                onclick={() => onToggle(hotkey)}
                class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 {hotkey.enabled
                  ? 'bg-primary-600'
                  : 'bg-gray-200'}"
                role="switch"
                aria-checked={hotkey.enabled}
                aria-label="Toggle hotkey {hotkey.name}"
              >
                <span
                  class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {hotkey.enabled
                    ? 'translate-x-5'
                    : 'translate-x-0'}"
                ></span>
              </button>
            </td>
            <td class="whitespace-nowrap px-6 py-4 text-right text-sm font-medium">
              <button
                onclick={() => onEdit(hotkey)}
                class="text-primary-600 hover:text-primary-900 mr-4"
              >
                Edit
              </button>
              <button onclick={() => onDelete(hotkey)} class="text-red-600 hover:text-red-900">
                Delete
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
