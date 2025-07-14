<script lang="ts">
	import { force_sync, open_folder } from '$lib/utils';
	import { invoke } from '@tauri-apps/api';
	import { Globe, RefreshCcw, FolderInput, Settings } from '@lucide/svelte';
	import { config } from '$lib/store.svelte';
</script>

<div class="mb-6 flex w-full items-center justify-between">
	<button
		onclick={() => window.open(config.server_url, '_blank')}
		class="cursor-pointer text-gray-600 transition-colors hover:text-gray-800"
		title="Abrir URL de la API"
	>
		<Globe />
	</button>

	<button
		class="cursor-pointer text-green-500 transition-colors hover:text-green-600"
		title="Synchronize"
		onclick={force_sync}
	>
		<RefreshCcw />
	</button>
	<button
		class="cursor-pointer text-gray-600 transition-colors hover:text-gray-800"
		title="Open Folder"
		onclick={open_folder}
	>
		<FolderInput />
	</button>

	<button
		class="cursor-pointer text-gray-600 transition-colors hover:text-gray-800"
		title="Configuration"
		onclick={() => invoke('open_config_window')}
	>
		<Settings />
	</button>
</div>
