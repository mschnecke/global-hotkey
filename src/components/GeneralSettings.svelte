<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { getAutostart, setAutostart } from '$lib/commands';

  let launchAtStartup = $state(false);
  let loading = $state(true);
  let version = $state('');

  onMount(async () => {
    try {
      [launchAtStartup, version] = await Promise.all([getAutostart(), getVersion()]);
    } catch (e) {
      console.error('Failed to load settings:', e);
    } finally {
      loading = false;
    }
  });

  async function handleToggleAutostart() {
    const newValue = !launchAtStartup;
    try {
      await setAutostart(newValue);
      launchAtStartup = newValue;
    } catch (e) {
      console.error('Failed to toggle autostart:', e);
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-gray-900">General Settings</h3>
    <p class="mt-1 text-sm text-gray-500">Configure application behavior</p>
  </div>

  <div class="space-y-4">
    <!-- Launch at Startup -->
    <div class="flex items-center justify-between">
      <div>
        <div class="text-sm font-medium text-gray-700">Launch at startup</div>
        <div class="text-xs text-gray-500">Start Global Hotkey when you log in</div>
      </div>
      <button
        onclick={handleToggleAutostart}
        disabled={loading}
        class="relative w-11 h-6 rounded-full transition-colors disabled:opacity-50 {launchAtStartup
          ? 'bg-primary-500'
          : 'bg-gray-300'}"
        aria-label="Toggle launch at startup"
      >
        <span
          class="absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform shadow {launchAtStartup
            ? 'translate-x-5'
            : ''}"
        ></span>
      </button>
    </div>
  </div>

  <!-- About -->
  <div class="pt-6 border-t border-gray-200">
    <div>
      <h3 class="text-lg font-medium text-gray-900">About</h3>
    </div>
    <div class="mt-2 text-sm text-gray-500">
      Global Hotkey v{version}
    </div>
  </div>
</div>
