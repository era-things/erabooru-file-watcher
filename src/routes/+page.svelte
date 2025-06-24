<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { load } from '@tauri-apps/plugin-store';

  let folder = $state('folder');
  let server = $state('server');
  let running = $state(false);

  async function selectFolder() {
    const selected = await open({ 
      directory: true,
      multiple: false 
    });
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

<div class="container">
  <div class="row">
    <label>
      Folder 
      <input readonly bind:value={folder} />
    </label>
    <button onclick={selectFolder}>Select</button>
  </div>
  <div class="row">
    <label>
      Server 
      <input bind:value={server} />
    </label>
  </div>
  <div class="row">
    <button onclick={saveState}>Save</button>
    <button onclick={toggle}>{running ? 'Stop' : 'Run'}</button>
  </div>
</div>

<style>
  .container {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .row {
    display: flex;
    align-items: center;
    
    gap: 0.5rem;
  }
  input {
    flex: 1;
    width: 220px
  }
</style>
