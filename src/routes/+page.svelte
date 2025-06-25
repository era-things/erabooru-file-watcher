<script lang="ts">
  import Tabs from '$lib/Tabs.svelte';
  import WatchTab from '$lib/WatchTab.svelte';
  import UploadTab from '$lib/UploadTab.svelte';

  let tab = $state<'watch' | 'upload'>('watch');
  
  // Single state objects for each tab
  let watchState = $state({
    folder: '',
    server: '',
    running: false
  });

  let uploadState = $state({
    folder: '',
    videoCount: 0,
    imageCount: 0,
    totalSize: 0
  });
</script>

<Tabs {tab} on:change={(e) => (tab = e.detail)} />

{#if tab === 'watch'}
  <WatchTab bind:state={watchState} />
{:else}
  <UploadTab bind:state={uploadState} />
{/if}