<script lang="ts">
	import { config } from '$lib/store.svelte';
	import { update_config } from '$lib/utils';
	let error = $state('');
</script>

<p class="mb-3 block text-sm font-medium text-gray-700">Server URL</p>
<div class="w-ful flex gap-2">
	<input
		pattern="https?://.+"
		required
		type="text"
		onchange={(e) => (e.target as HTMLInputElement).reportValidity()}
		bind:value={config.server_url}
		class="grow rounded-md border border-gray-300 px-3 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 focus:outline-none"
		placeholder="Server URL ex. http://127.0.0.1:8080"
	/>
	<button
		onclick={() => {
			error = '';
			update_config().catch((e) => (error = e.server));
		}}
		class="rounded-md bg-blue-600 px-4 py-2 text-white transition-colors duration-200 hover:bg-blue-700"
	>
		Save
	</button>
</div>
<p class:invisible={!error} class=" mt-2 text-sm text-red-600">{error}</p>
