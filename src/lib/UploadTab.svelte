<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';

  let folder = $state('');
  let videoCount = $state(0);
  let imageCount = $state(0);
  let totalSize = $state(0);

  async function selectFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      folder = selected as string;
      const result = await invoke<[number, number, number]>('scan_folder', { folder });
      videoCount = result[0];
      imageCount = result[1];
      totalSize = result[2];
    }
  }

  function formatSize(bytes: number) {
    return (bytes / 1024 / 1024).toFixed(2);
  }

  async function upload() {
    await invoke('upload_folder', { folder });
  }
</script>

<div class="p-4 space-y-4">
  <div class="flex items-center gap-2">
    <label class="flex-1">
      <span class="text-sm">Folder</span>
      <input class="w-full border p-1" readonly bind:value={folder} />
    </label>
    <button class="px-3 py-1 rounded bg-blue-500 text-white" onclick={selectFolder}>Select</button>
  </div>
  {#if folder}
    <div>
      <span>
        This will upload {videoCount} videos and {imageCount} images, with total size of {formatSize(totalSize)} MB
      </span>
    </div>
    <div>
      <button class="px-3 py-1 rounded bg-blue-500 text-white" onclick={upload}>Upload</button>
    </div>
  {/if}
</div>

