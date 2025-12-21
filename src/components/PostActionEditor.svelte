<script lang="ts">
  import type { PostActionsConfig, PostAction, PostActionType } from '$lib/types';
  import HotkeyRecorder from './HotkeyRecorder.svelte';

  interface Props {
    value: PostActionsConfig;
    onChange: (config: PostActionsConfig) => void;
  }

  let { value, onChange }: Props = $props();

  function createDefaultConfig(): PostActionsConfig {
    return {
      enabled: false,
      trigger: { type: 'onExit' },
      actions: [],
    };
  }

  function addAction(type: 'pasteClipboard' | 'simulateKeystroke' | 'delay') {
    const config = value || createDefaultConfig();
    const newAction: PostAction = {
      id: crypto.randomUUID(),
      actionType: createActionType(type),
      enabled: true,
    };
    onChange({
      ...config,
      actions: [...config.actions, newAction],
    });
  }

  function createActionType(
    type: 'pasteClipboard' | 'simulateKeystroke' | 'delay'
  ): PostActionType {
    switch (type) {
      case 'pasteClipboard':
        return { type: 'pasteClipboard' };
      case 'simulateKeystroke':
        return { type: 'simulateKeystroke', keystroke: { modifiers: [], key: '' } };
      case 'delay':
        return { type: 'delay', delayMs: 500 };
    }
  }

  function updateAction(index: number, action: PostAction) {
    const newActions = [...value.actions];
    newActions[index] = action;
    onChange({ ...value, actions: newActions });
  }

  function removeAction(index: number) {
    onChange({
      ...value,
      actions: value.actions.filter((_, i) => i !== index),
    });
  }

  function moveAction(index: number, direction: 'up' | 'down') {
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= value.actions.length) return;

    const newActions = [...value.actions];
    [newActions[index], newActions[newIndex]] = [newActions[newIndex], newActions[index]];
    onChange({ ...value, actions: newActions });
  }
</script>

<div class="space-y-4 border-t pt-4 mt-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-gray-700">Post-Actions</h3>
    <label class="flex items-center">
      <input
        type="checkbox"
        checked={value?.enabled || false}
        onchange={(e) =>
          onChange({ ...(value || createDefaultConfig()), enabled: e.currentTarget.checked })}
        class="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500"
      />
      <span class="ml-2 text-sm text-gray-600">Enable</span>
    </label>
  </div>

  {#if value?.enabled}
    <!-- Trigger selection -->
    <div>
      <label for="post-action-trigger" class="block text-sm font-medium text-gray-700"
        >Trigger</label
      >
      <select
        id="post-action-trigger"
        class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
        value={value.trigger?.type || 'onExit'}
        onchange={(e) => {
          const type = e.currentTarget.value;
          onChange({
            ...value,
            trigger: type === 'onExit' ? { type: 'onExit' } : { type: 'afterDelay', delayMs: 1000 },
          });
        }}
      >
        <option value="onExit">After process exits (exit code 0)</option>
        <option value="afterDelay">After delay from launch</option>
      </select>

      {#if value.trigger?.type === 'afterDelay'}
        <div class="mt-2 flex items-center gap-2">
          <input
            type="number"
            min="0"
            step="100"
            value={value.trigger.delayMs}
            onchange={(e) =>
              onChange({
                ...value,
                trigger: { type: 'afterDelay', delayMs: parseInt(e.currentTarget.value) || 0 },
              })}
            class="w-24 rounded-md border border-gray-300 px-3 py-2 text-sm shadow-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
          <span class="text-sm text-gray-500">ms</span>
        </div>
      {/if}
    </div>

    <!-- Actions list -->
    <div class="space-y-2">
      <span class="block text-sm font-medium text-gray-700">Actions</span>

      {#each value.actions as action, index}
        <div class="flex items-center gap-2 rounded-md border border-gray-200 bg-gray-50 p-3">
          <!-- Reorder buttons -->
          <div class="flex flex-col gap-1">
            <button
              type="button"
              onclick={() => moveAction(index, 'up')}
              disabled={index === 0}
              class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
              aria-label="Move action up"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 15l7-7 7 7"
                />
              </svg>
            </button>
            <button
              type="button"
              onclick={() => moveAction(index, 'down')}
              disabled={index === value.actions.length - 1}
              class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-30"
              aria-label="Move action down"
            >
              <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>
          </div>

          <!-- Action configuration -->
          <div class="flex-1">
            {#if action.actionType.type === 'pasteClipboard'}
              <span class="text-sm font-medium">Paste Clipboard</span>
              <span class="text-xs text-gray-500 ml-2">(Ctrl/Cmd + V)</span>
            {:else if action.actionType.type === 'simulateKeystroke'}
              <div class="flex items-center gap-2">
                <span class="text-sm">Keystroke:</span>
                <HotkeyRecorder
                  value={{
                    modifiers: action.actionType.keystroke.modifiers,
                    key: action.actionType.keystroke.key,
                  }}
                  onCapture={(hk) =>
                    updateAction(index, {
                      ...action,
                      actionType: { type: 'simulateKeystroke', keystroke: hk },
                    })}
                />
              </div>
            {:else if action.actionType.type === 'delay'}
              <div class="flex items-center gap-2">
                <span class="text-sm">Wait</span>
                <input
                  type="number"
                  min="0"
                  step="100"
                  value={action.actionType.delayMs}
                  onchange={(e) =>
                    updateAction(index, {
                      ...action,
                      actionType: { type: 'delay', delayMs: parseInt(e.currentTarget.value) || 0 },
                    })}
                  class="w-20 rounded-md border border-gray-300 px-2 py-1 text-sm"
                />
                <span class="text-sm text-gray-500">ms</span>
              </div>
            {/if}
          </div>

          <!-- Enable/disable toggle -->
          <input
            type="checkbox"
            checked={action.enabled}
            onchange={(e) => updateAction(index, { ...action, enabled: e.currentTarget.checked })}
            class="h-4 w-4 rounded border-gray-300 text-primary-600"
            title="Enable/disable this action"
          />

          <!-- Remove button -->
          <button
            type="button"
            onclick={() => removeAction(index)}
            class="p-1 text-red-400 hover:text-red-600"
            title="Remove action"
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
      {/each}

      <!-- Add action buttons -->
      <div class="flex gap-2 pt-2">
        <button
          type="button"
          onclick={() => addAction('pasteClipboard')}
          class="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          + Paste
        </button>
        <button
          type="button"
          onclick={() => addAction('simulateKeystroke')}
          class="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          + Keystroke
        </button>
        <button
          type="button"
          onclick={() => addAction('delay')}
          class="rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          + Delay
        </button>
      </div>
    </div>
  {/if}
</div>
