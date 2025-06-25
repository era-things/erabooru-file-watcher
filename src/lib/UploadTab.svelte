<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import FolderPicker from '../components/FolderPicker.svelte';

  interface UploadState {
    folder: string;
    videoCount: number;
    imageCount: number;
    totalSize: number;
  }

  interface Props {
    state: UploadState;
  }

  let { state = $bindable() }: Props = $props();

  let hasMedia = $derived(state.videoCount > 0 || state.imageCount > 0);

  async function onFolderChange(selectedFolder: string) {
    if (selectedFolder) {
      const result = await invoke<[number, number, number]>('scan_folder', { folder: selectedFolder });
      state.videoCount = result[0];
      state.imageCount = result[1];
      state.totalSize = result[2];
    }
  }

  function formatSize(bytes: number) {
    return (bytes / 1024 / 1024).toFixed(2);
  }

  function getMediaDescription() {
    const hasVideos = state.videoCount > 0;
    const hasImages = state.imageCount > 0;
    
    if (!hasVideos && !hasImages) {
      return "This folder does not contain any supported media files.";
    }
    
    const parts = [];
    if (hasVideos) {
      parts.push(`<span class="font-semibold text-blue-600">${state.videoCount} video${state.videoCount !== 1 ? 's' : ''}</span>`);
    }
    if (hasImages) {
      parts.push(`<span class="font-semibold text-green-600">${state.imageCount} image${state.imageCount !== 1 ? 's' : ''}</span>`);
    }
    
    const mediaText = parts.length === 2 ? parts.join(' and ') : parts[0];
    return `This will upload ${mediaText}, with total size of <span class="font-semibold text-purple-600">${formatSize(state.totalSize)} MB</span>`;
  }

  async function upload() {
    await invoke('upload_folder', { folder: state.folder });
  }
</script>

<div class="p-4 space-y-4">
  <FolderPicker bind:value={state.folder} label="Upload Folder" onchange={onFolderChange} />
  
  {#if state.folder}
    <div class="p-4 bg-gray-50 rounded-lg">
      <p class="text-sm text-gray-700">
        {@html getMediaDescription()}
      </p>
    </div>
    {#if hasMedia}
      <div>
        <button 
          class="px-4 py-2 rounded bg-orange-500 hover:bg-orange-600 text-white text-sm font-medium transition-colors" 
          onclick={upload}
        >
          Start Upload
        </button>
      </div>
    {/if}
  {/if}
</div>