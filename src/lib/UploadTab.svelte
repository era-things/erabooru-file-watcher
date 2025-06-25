<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import FolderPicker from '../components/FolderPicker.svelte';

  let folder = $state('');
  let videoCount = $state(0);
  let imageCount = $state(0);
  let totalSize = $state(0);

  let hasMedia = $derived(videoCount > 0 || imageCount > 0);

  async function onFolderChange(selectedFolder: string) {
    if (selectedFolder) {
      const result = await invoke<[number, number, number]>('scan_folder', { folder: selectedFolder });
      videoCount = result[0];
      imageCount = result[1];
      totalSize = result[2];
    }
  }

  function formatSize(bytes: number) {
    return (bytes / 1024 / 1024).toFixed(2);
  }

  function getMediaDescription() {
    const hasVideos = videoCount > 0;
    const hasImages = imageCount > 0;
    
    if (!hasVideos && !hasImages) {
      return "This folder does not contain any supported media files.";
    }
    
    const parts = [];
    if (hasVideos) {
      parts.push(`<span class="font-semibold text-blue-600">${videoCount} video${videoCount !== 1 ? 's' : ''}</span>`);
    }
    if (hasImages) {
      parts.push(`<span class="font-semibold text-green-600">${imageCount} image${imageCount !== 1 ? 's' : ''}</span>`);
    }
    
    const mediaText = parts.length === 2 ? parts.join(' and ') : parts[0];
    return `This will upload ${mediaText} files, with total size of <span class="font-semibold text-purple-600">${formatSize(totalSize)} MB</span>`;
  }

  async function upload() {
    await invoke('upload_folder', { folder });
  }
</script>

<div class="p-4 space-y-4">
  <FolderPicker bind:value={folder} label="Upload Folder" onchange={onFolderChange} />
  
  {#if folder}
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