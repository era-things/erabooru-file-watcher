<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { load } from '@tauri-apps/plugin-store';
  import { onMount } from 'svelte';

  let folder = $state('');
  let server = $state('');
  let running = $state(false);

  async function selectFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) folder = selected as string;
  }

  type Settings = { folder: string; server: string };

  async function loadState() {
    const store = await load('store.json');
    const settings = await store.get<Settings>('settings');
    if (settings) {
      folder = settings.folder || '';
      server = settings.server || '';
    }
  }

  async function saveState() {
    const store = await load('store.json');
    await store.set('settings', { folder, server });
    await store.save();
  }

  async function toggle() {
    if (!running) {
      await invoke('start_watching');
      running = true;
    } else {
      await invoke('stop_watching');
      running = false;
    }
  }

  onMount(loadState);
</script>

<div class="p-4 space-y-4">
  <div class="flex items-center gap-2">
    <label class="flex-1">
      <span class="text-sm">Folder</span>
      <input class="w-full border p-1" readonly bind:value={folder} />
    </label>
    <button class="px-3 py-1 rounded bg-blue-500 text-white" onclick={selectFolder}>
      Select
    </button>
  </div>
  <div class="flex items-center gap-2">
    <label class="flex-1">
      <span class="text-sm">Server</span>
      <input class="w-full border p-1" bind:value={server} />
    </label>
  </div>
  <div class="flex items-center gap-2">
    <button class="px-3 py-1 rounded bg-blue-500 text-white" onclick={saveState}>Save</button>
    <button class="px-3 py-1 rounded bg-blue-500 text-white" onclick={toggle}>
      {running ? 'Stop' : 'Run'}
    </button>
  </div>
</div>

