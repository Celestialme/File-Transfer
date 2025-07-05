<script lang="ts">
	import { save_initial_config } from '$lib/utils';
	import { open } from '@tauri-apps/api/dialog';
	let isLoading = $state(false);
	let serverUrl = $state('');
	let folderPath = $state('');
	async function submit(e: Event) {
		e.preventDefault();
		isLoading = true;
		await save_initial_config({ serverUrl, folderPath });
		isLoading = false;
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-100">
	<div class="w-full max-w-md rounded-lg bg-white p-8 shadow-lg">
		<h1 class="mb-8 text-center text-2xl font-bold">Configuración Inicial</h1>

		<form class="space-y-6" onsubmit={submit}>
			<div>
				<div class="flex items-center">
					<div class="mr-3">
						<svg
							class="h-6 w-6 text-gray-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width={2}
								d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
							/>
						</svg>
					</div>
					<p class="block text-sm font-medium text-gray-700">URL del servidor</p>
				</div>
				<input
					type="url"
					bind:value={serverUrl}
					placeholder="https://mi-servidor.com"
					class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 focus:outline-none"
					required
					disabled={isLoading}
				/>
			</div>

			<div>
				<div class="flex items-center">
					<div class="mr-3">
						<svg
							class="h-6 w-6 text-gray-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width={2}
								d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
							/>
						</svg>
					</div>
					<p class="block text-sm font-medium text-gray-700">Carpeta de sincronización</p>
				</div>
				<div class="mt-1 flex rounded-md shadow-sm">
					<input
						type="text"
						bind:value={folderPath}
						placeholder="Selecciona una carpeta"
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
						Examinar
					</button>
				</div>
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
					<div class="flex items-center justify-center">
						<svg
							class="mr-3 -ml-1 h-5 w-5 animate-spin text-gray-700"
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
						>
							<circle
								class="opacity-25"
								cx="12"
								cy="12"
								r="10"
								stroke="currentColor"
								stroke-width="4"
							></circle>
							<path
								class="opacity-75"
								fill="currentColor"
								d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
							></path>
						</svg>
						Verificando...
					</div>
				{:else}
					Guardar y continuar
				{/if}
			</button>
		</form>
	</div>
</div>
