<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { load } from '@tauri-apps/plugin-store';

  let folder = $state('folder');
  let server = $state('server');
  let running = $state(false);
  let tab: 'watch' | 'upload' = $state('watch');

  // upload tab state
  let uploadFolder = $state('');
  let videoCount = $state(0);
  let imageCount = $state(0);
  let totalSize = $state(0);

  async function selectFolder() {
    const selected = await open({
      directory: true,
      multiple: false
    });
    if (selected) folder = selected as string;
  }

  async function selectUploadFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      uploadFolder = selected as string;
      const result = await invoke<[number, number, number]>('scan_folder', { folder: uploadFolder });
      videoCount = result[0];
      imageCount = result[1];
      totalSize = result[2];
    }
  }

  function formatSize(bytes: number) {
    return (bytes / 1024 / 1024).toFixed(2);
  }

  async function upload() {
    await invoke('upload_folder', { folder: uploadFolder });
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

<div class="tabs">
  <button class:active={tab === 'watch'} onclick={() => (tab = 'watch')}>Watch</button>
  <button class:active={tab === 'upload'} onclick={() => (tab = 'upload')}>Upload</button>
</div>

{#if tab === 'watch'}
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
{:else if tab === 'upload'}
  <div class="container">
    <div class="row">
      <label>
        Folder
        <input readonly bind:value={uploadFolder} />
      </label>
      <button onclick={selectUploadFolder}>Select</button>
    </div>
    {#if uploadFolder}
      <div class="row">
        <span>This will upload {videoCount} videos and {imageCount} images, with total size of {formatSize(totalSize)} MB</span>
      </div>
      <div class="row">
        <button onclick={upload}>Upload</button>
      </div>
    {/if}
  </div>
{/if}

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
  .tabs {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem;
  }
  .tabs button.active {
    font-weight: bold;
  }
</style>
