<script lang="ts">
  import { onMount } from 'svelte';
  import { load } from '@tauri-apps/plugin-store';

  interface Pair {
    folder: string;
    tags: string;
  }

  let pairs = $state<Pair[]>([]);
  let overrideUploadDate = $state(false);

  async function loadPairs() {
    const store = await load('store.json');
    const settings = await store.get<any>('settings');
    pairs = settings?.auto_tags ?? [];
    overrideUploadDate = settings?.override_upload_date ?? false;
  }

  async function savePairs() {
    const store = await load('store.json');
    const settings = await store.get<any>('settings') ?? {};
    settings.auto_tags = pairs;
    settings.override_upload_date = overrideUploadDate;
    await store.set('settings', settings);
    await store.save();
  }

  function addPair() {
    pairs = [...pairs, { folder: '', tags: '' }];
  }

  function removePair(index: number) {
    pairs = pairs.filter((_, i) => i !== index);
  }

  onMount(loadPairs);
</script>

<div class="p-4 space-y-4">
  <div class="flex items-center gap-2">
    <input type="checkbox" bind:checked={overrideUploadDate} id="override-date" />
    <label for="override-date" class="text-sm">Override upload date with system date</label>
  </div>
  <div class="space-y-2">
    {#each pairs as pair, i}
      <div class="flex gap-2 items-center">
        <input class="flex-1 border border-gray-300 rounded px-3 py-2 text-sm" bind:value={pair.folder} placeholder="Folder word" />
        <input class="flex-1 border border-gray-300 rounded px-3 py-2 text-sm" bind:value={pair.tags} placeholder="tags" />
        <button class="px-2 py-1 rounded bg-red-500 hover:bg-red-600 text-white text-xs" on:click={() => removePair(i)}>X</button>
      </div>
    {/each}
  </div>
  <button class="px-3 py-1 rounded bg-gray-200 text-sm" on:click={addPair}>Add Pair</button>
  <div>
    <button class="px-4 py-2 rounded bg-green-500 hover:bg-green-600 text-white text-sm" on:click={savePairs}>Save</button>
  </div>
</div>
