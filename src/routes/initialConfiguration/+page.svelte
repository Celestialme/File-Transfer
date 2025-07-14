<script lang="ts">
	import { save_initial_config } from '$lib/utils';
	import { Folder, Globe, LoaderCircle } from '@lucide/svelte';
	import { open } from '@tauri-apps/api/dialog';
	let isLoading = $state(false);
	let serverUrl = $state('');
	let folderPath = $state('');
	let error = $state({
		server: '',
		folder: ''
	});
	async function submit(e: Event) {
		e.preventDefault();
		isLoading = true;
		await save_initial_config({ serverUrl, folderPath }).catch((e) => {
			console.log(e);
			error = e;
		});
		isLoading = false;
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-100">
	<div class="w-full max-w-md rounded-lg bg-white p-8 shadow-lg">
		<h1 class="mb-8 text-center text-2xl font-bold">Initial Configuration</h1>

		<form class="space-y-6" onsubmit={submit}>
			<div>
				<div class="flex items-center">
					<div class="mr-3">
						<Globe class="h-6 w-6 text-gray-600" />
					</div>
					<p class="block text-sm font-medium text-gray-700">Server URL</p>
				</div>
				<input
					type="url"
					bind:value={serverUrl}
					placeholder="https://mi-servidor.com"
					class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 focus:outline-none"
					required
					disabled={isLoading}
				/>
				<p class:invisible={!error.server} class=" mt-2 text-sm text-red-600">{error.server}</p>
			</div>

			<div>
				<div class="flex items-center">
					<div class="mr-3">
						<Folder class="h-6 w-6 text-gray-600" />
					</div>
					<p class="block text-sm font-medium text-gray-700">Folder Path</p>
				</div>
				<div class="mt-1 flex rounded-md shadow-sm">
					<input
						type="text"
						bind:value={folderPath}
						placeholder="select local folder"
						class="block w-full rounded-l-md border border-r-0 border-gray-300 px-3 py-2 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 focus:outline-none"
						required
						disabled={isLoading}
					/>
					<button
						type="button"
						class="inline-flex items-center rounded-r-md border border-l-0 border-gray-300 bg-gray-50 px-3 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 focus:outline-none"
						disabled={isLoading}
						onclick={async () => {
							folderPath = (await open({
								directory: true,
								multiple: false
							})) as string;
						}}
					>
						Search
					</button>
				</div>
				<p class:invisible={!error.folder} class=" mt-2 text-sm text-red-600">{error.folder}</p>
			</div>

			<button
				type="submit"
				class={`w-full rounded-md px-4 py-2 text-sm font-medium focus:ring-2 focus:ring-offset-2 focus:outline-none ${
					isLoading
						? 'cursor-not-allowed bg-gray-300 text-gray-500'
						: 'bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-400'
				}`}
				disabled={isLoading}
			>
				{#if isLoading}
					<div class="flex items-center justify-center gap-2">
						<LoaderCircle class="animate-spin" />
						Verifying...
					</div>
				{:else}
					Save and Continue
				{/if}
			</button>
		</form>
	</div>
</div>
