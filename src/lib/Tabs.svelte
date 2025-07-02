<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let tab: 'watch' | 'upload' | 'settings';
  const dispatch = createEventDispatcher<{ change: 'watch' | 'upload' | 'settings' }>();
</script>

<div class="flex border-b mb-2">
  <button
    class={`px-4 py-2 -mb-px border-b-2 ${tab === 'watch' ? 'border-blue-500 text-blue-500 font-semibold' : 'border-transparent text-gray-500'}`}
    on:click={() => dispatch('change', 'watch')}
  >
    Watch
  </button>
  <button
    class={`px-4 py-2 -mb-px border-b-2 ${tab === 'upload' ? 'border-blue-500 text-blue-500 font-semibold' : 'border-transparent text-gray-500'}`}
    on:click={() => dispatch('change', 'upload')}
  >
    Upload
  </button>
  <button
    class={`px-4 py-2 -mb-px border-b-2 ${tab === 'settings' ? 'border-blue-500 text-blue-500 font-semibold' : 'border-transparent text-gray-500'}`}
    on:click={() => dispatch('change', 'settings')}
  >
    Settings
  </button>
</div>
