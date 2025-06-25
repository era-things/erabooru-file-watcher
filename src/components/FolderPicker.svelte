<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    value: string;
    placeholder?: string;
    label?: string;
    onchange?: (value: string) => void;
  }

  let { value = $bindable(), placeholder = "No folder selected", label = "Folder", onchange }: Props = $props();

  async function selectFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      value = selected as string;
      onchange?.(value);
    }
  }
</script>

<div class="space-y-1">
  {#if label}
    <span class="text-sm font-medium text-gray-700">{label}</span>
  {/if}
  <div class="flex items-stretch gap-2">
    <input 
      class="flex-1 border border-gray-300 rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" 
      readonly 
      bind:value 
      {placeholder}
    />
    <button 
      class="px-4 py-2 rounded bg-blue-500 hover:bg-blue-600 text-white text-sm font-medium transition-colors whitespace-nowrap" 
      onclick={selectFolder}
    >
      Select
    </button>
  </div>
</div>