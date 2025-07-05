<script lang="ts">
	import { config } from '$lib/store.svelte';
	import { update_config } from '$lib/utils';
	import { open } from '@tauri-apps/api/dialog';
	let is_changed = $state(false);
	let saved_folder_path = $state(config.folder_path || '');
	$effect(() => {
		is_changed = saved_folder_path != config.folder_path;
	});
	async function handleFolderSelect() {
		if (is_changed) {
			update_config();
			saved_folder_path = config.folder_path || '';
			return;
		}
		config.folder_path = (await open({
			directory: true,
			multiple: false
		})) as string;
	}
</script>

<p class="mb-3 block text-sm font-medium text-gray-700">Selección de carpeta local observada</p>
<div class="w-ful flex gap-2">
	<input
		type="text"
		bind:value={config.folder_path}
		class="grow rounded-md border border-gray-300 px-3 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 focus:outline-none"
		placeholder="Cambiar carpeta"
	/>
	<button
		onclick={handleFolderSelect}
		class="rounded-md bg-blue-600 px-4 py-2 text-white transition-colors duration-200 hover:bg-blue-700"
	>
		{#if is_changed}
			Save
		{:else}
			Seleccionar
		{/if}
	</button>
</div>
