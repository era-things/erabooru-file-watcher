<script lang="ts">
  import { open } from '@tauri-apps/api/dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let folder = '';
  let server = '';
  let running = false;

  async function selectFolder() {
    const selected = await open({ directory: true });
    if (selected) folder = selected as string;
  }

  type Settings = { folder: string; server: string };
  async function load() {
    const settings = await invoke<Settings>('load_settings');
    if (settings) {
      folder = settings.folder || '';
      server = settings.server || '';
    }
  }

  async function save() {
    await invoke('save_settings', { folder, server });
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

  onMount(load);
</script>

<div class="container">
  <div class="row">
    <label>
      Folder
      <input readonly bind:value={folder} />
    </label>
    <button on:click={selectFolder}>Select</button>
  </div>
  <div class="row">
    <label>
      Server
      <input bind:value={server} />
    </label>
  </div>
  <div class="row">
    <button on:click={save}>Save</button>
    <button on:click={toggle}>{running ? 'Stop' : 'Run'}</button>
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
  }
</style>
