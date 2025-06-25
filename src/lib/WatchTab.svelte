<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { load } from '@tauri-apps/plugin-store';
  import { onMount } from 'svelte';
  import FolderPicker from '../components/FolderPicker.svelte';

  interface WatchState {
    folder: string;
    server: string;
    running: boolean;
  }

  interface Props {
    state: WatchState;
  }

  let { state = $bindable() }: Props = $props();

  type Settings = { folder: string; server: string };

  async function loadState() {
    const store = await load('store.json');
    const settings = await store.get<Settings>('settings');
    if (settings) {
      state.folder = settings.folder || '';
      state.server = settings.server || '';
    }
  }

  async function saveState() {
    const store = await load('store.json');
    await store.set('settings', { folder: state.folder, server: state.server });
    await store.save();
  }

  async function toggle() {
    if (!state.running) {
      await invoke('start_watching');
      state.running = true;
    } else {
      await invoke('stop_watching');
      state.running = false;
    }
  }

  onMount(loadState);
</script>

<div class="p-4 space-y-4">
  <FolderPicker bind:value={state.folder} label="Watch Folder" />
  
  <div class="space-y-1">
    <span class="text-sm font-medium text-gray-700">Server</span>
    <input 
      class="w-full border border-gray-300 rounded px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" 
      bind:value={state.server} 
      placeholder="http://localhost:3000"
    />
  </div>
  
  <div class="flex items-center gap-2">
    <button 
      class="px-4 py-2 rounded bg-green-500 hover:bg-green-600 text-white text-sm font-medium transition-colors" 
      onclick={saveState}
    >
      Save
    </button>
    <button 
      class="px-4 py-2 rounded text-sm font-medium transition-colors" 
      class:bg-red-500={state.running}
      class:hover:bg-red-600={state.running}
      class:bg-blue-500={!state.running}
      class:hover:bg-blue-600={!state.running}
      class:text-white={true}
      onclick={toggle}
    >
      {state.running ? 'Stop' : 'Start Watching'}
    </button>
  </div>
</div>