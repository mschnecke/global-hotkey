<script lang="ts">
  import type { AiSettings, AiProviderConfig, AiRole } from '$lib/types';
  import { testAiProvider, getBuiltinRoles, saveAiRole, deleteAiRole } from '$lib/commands';
  import { onMount } from 'svelte';
  import RoleEditor from './RoleEditor.svelte';

  interface Props {
    value: AiSettings;
    onChange: (settings: AiSettings) => void;
  }

  let { value, onChange }: Props = $props();

  let testStatus: 'idle' | 'testing' | 'success' | 'error' = $state('idle');
  let testError: string = $state('');
  let showApiKey: boolean = $state(false);

  // Role editor state
  let roleEditorOpen = $state(false);
  let editingRole = $state<AiRole | null>(null);

  const VALID_MODELS = ['gemini-2.5-flash-lite', 'gemini-2.5-pro'];
  const DEFAULT_MODEL = 'gemini-2.5-flash-lite';

  // Initialize with defaults if empty or fix invalid model
  onMount(async () => {
    if (!value.providers || value.providers.length === 0) {
      onChange({
        ...value,
        providers: [
          {
            id: crypto.randomUUID(),
            providerType: 'gemini',
            apiKey: '',
            model: DEFAULT_MODEL,
            enabled: true,
          },
        ],
      });
    } else {
      // Fix invalid model names from old config
      const currentModel = value.providers[0]?.model;
      if (currentModel && !VALID_MODELS.includes(currentModel)) {
        updateProvider({ model: DEFAULT_MODEL });
      }
    }

    // Load built-in roles if empty
    if (!value.roles || value.roles.length === 0) {
      try {
        const builtinRoles = await getBuiltinRoles();
        onChange({
          ...value,
          roles: builtinRoles,
        });
      } catch (e) {
        console.error('Failed to load built-in roles:', e);
      }
    }
  });

  async function handleTest() {
    const provider = value.providers?.[0];
    if (!provider?.apiKey) {
      testError = 'Please enter an API key';
      testStatus = 'error';
      return;
    }

    testStatus = 'testing';
    testError = '';

    try {
      await testAiProvider(provider.apiKey, provider.model);
      testStatus = 'success';
    } catch (e) {
      testStatus = 'error';
      testError = String(e);
    }
  }

  function updateProvider(updates: Partial<AiProviderConfig>) {
    const providers = [...(value.providers || [])];
    if (providers.length === 0) {
      providers.push({
        id: crypto.randomUUID(),
        providerType: 'gemini',
        apiKey: '',
        enabled: true,
      });
    }
    providers[0] = { ...providers[0], ...updates };
    onChange({ ...value, providers });
  }

  function openAddRole() {
    editingRole = null;
    roleEditorOpen = true;
  }

  function openEditRole(role: AiRole) {
    editingRole = role;
    roleEditorOpen = true;
  }

  async function handleSaveRole(role: AiRole) {
    try {
      await saveAiRole(role);
      // Update local state
      const roles = [...(value.roles || [])];
      const existingIdx = roles.findIndex((r) => r.id === role.id);
      if (existingIdx >= 0) {
        roles[existingIdx] = role;
      } else {
        roles.push(role);
      }
      onChange({ ...value, roles });
      roleEditorOpen = false;
    } catch (e) {
      console.error('Failed to save role:', e);
      alert('Failed to save role: ' + e);
    }
  }

  async function handleDeleteRole(role: AiRole) {
    if (role.isBuiltin) {
      alert('Cannot delete built-in roles');
      return;
    }

    if (!confirm(`Delete role "${role.name}"?`)) {
      return;
    }

    try {
      await deleteAiRole(role.id);
      // Update local state
      const roles = (value.roles || []).filter((r) => r.id !== role.id);
      onChange({ ...value, roles });
    } catch (e) {
      console.error('Failed to delete role:', e);
      alert('Failed to delete role: ' + e);
    }
  }
</script>

<div class="space-y-6">
  <div>
    <h3 class="text-lg font-medium text-gray-900 mb-4">AI Provider</h3>

    <div class="space-y-4 p-4 bg-gray-50 rounded-lg">
      <div class="flex items-center gap-2">
        <span class="font-medium text-gray-700">Gemini</span>
        {#if testStatus === 'success'}
          <span class="text-green-600 text-sm">Connected</span>
        {/if}
      </div>

      <div>
        <label for="api-key" class="block text-sm font-medium text-gray-700"> API Key </label>
        <div class="mt-1 flex gap-2">
          <input
            id="api-key"
            type={showApiKey ? 'text' : 'password'}
            value={value.providers?.[0]?.apiKey || ''}
            oninput={(e) => updateProvider({ apiKey: e.currentTarget.value })}
            placeholder="AIza..."
            class="flex-1 rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
          <button
            type="button"
            onclick={() => (showApiKey = !showApiKey)}
            class="px-3 py-2 text-sm border border-gray-300 rounded-md hover:bg-gray-100"
          >
            {showApiKey ? 'Hide' : 'Show'}
          </button>
          <button
            type="button"
            onclick={handleTest}
            disabled={testStatus === 'testing'}
            class="px-3 py-2 text-sm bg-primary-600 text-white rounded-md hover:bg-primary-700 disabled:opacity-50"
          >
            {testStatus === 'testing' ? 'Testing...' : 'Test'}
          </button>
        </div>
        {#if testError}
          <p class="mt-1 text-sm text-red-600">{testError}</p>
        {/if}
        <p class="mt-1 text-xs text-gray-500">
          Get your API key from <a
            href="https://aistudio.google.com/apikey"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary-600 hover:underline">Google AI Studio</a
          >
        </p>
      </div>

      <div>
        <label for="model" class="block text-sm font-medium text-gray-700"> Model </label>
        <select
          id="model"
          value={value.providers?.[0]?.model || 'gemini-2.5-flash-lite'}
          onchange={(e) => updateProvider({ model: e.currentTarget.value })}
          class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
        >
          <option value="gemini-2.5-flash-lite">gemini-2.5-flash-lite (Fast)</option>
          <option value="gemini-2.5-pro">gemini-2.5-pro (Quality)</option>
        </select>
      </div>
    </div>
  </div>

  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-medium text-gray-900">AI Roles</h3>
      <button
        type="button"
        onclick={openAddRole}
        class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-md hover:bg-primary-700"
      >
        Add Role
      </button>
    </div>
    <p class="text-sm text-gray-500 mb-2">
      Configure AI roles with custom system prompts for different tasks.
    </p>
    <div class="space-y-2">
      {#each value.roles || [] as role}
        <div class="p-3 bg-gray-50 rounded-md flex items-start justify-between gap-2">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="font-medium text-sm">{role.name}</span>
              {#if role.isBuiltin}
                <span class="text-xs bg-gray-200 text-gray-600 px-1.5 py-0.5 rounded">Built-in</span
                >
              {/if}
            </div>
            <div class="text-xs text-gray-500 truncate">{role.systemPrompt}</div>
          </div>
          <div class="flex gap-1 shrink-0">
            <button
              type="button"
              onclick={() => openEditRole(role)}
              class="p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded"
              title="Edit role"
            >
              <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                />
              </svg>
            </button>
            {#if !role.isBuiltin}
              <button
                type="button"
                onclick={() => handleDeleteRole(role)}
                class="p-1 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded"
                title="Delete role"
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                  />
                </svg>
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<RoleEditor
  open={roleEditorOpen}
  role={editingRole}
  onSave={handleSaveRole}
  onClose={() => (roleEditorOpen = false)}
/>
