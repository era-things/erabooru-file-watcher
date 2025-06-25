<script lang="ts">
  import Tabs from '$lib/Tabs.svelte';
  import WatchTab from '$lib/WatchTab.svelte';
  import UploadTab from '$lib/UploadTab.svelte';

  let tab: 'watch' | 'upload' = 'watch';
</script>

<Tabs {tab} on:change={(e) => (tab = e.detail)} />

{#if tab === 'watch'}
  <WatchTab />
{:else}
  <UploadTab />
{/if}

