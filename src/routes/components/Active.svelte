<script lang="ts">
	import Progress from './Progress.svelte';

	let {
		transfers
	}: {
		transfers: {
			path: string;
			progress: number;
			state: 'active' | 'completed';
			type: 'download' | 'upload';
		}[];
	} = $props();
</script>

<div class="max-h-[225px] w-full overflow-auto py-12 text-center">
	{#if transfers.length > 0}
		<div class="flex flex-col gap-2">
			{#each transfers as active}
				<div class="rounded-lg border border-gray-300 px-4 py-2">
					<div class="flex w-full items-center gap-2">
						<p
							class="max-w-[180px] min-w-[180px] overflow-hidden text-start overflow-ellipsis text-gray-600"
						>
							{active.path.split(/\/|\\/).pop()}
						</p>
						<Progress progress={active.progress} />
					</div>
					<p class="mt-2 text-gray-400">{active.path}</p>
				</div>
			{/each}
		</div>
	{:else}
		<div class="mb-4 text-gray-400">
			<svg class="mx-auto h-16 w-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width={2}
					d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
				/>
			</svg>
		</div>
		<p class="text-lg text-gray-500">No hay transferencias en curso</p>
	{/if}
</div>
