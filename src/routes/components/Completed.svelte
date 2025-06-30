<script lang="ts">
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
			{#each transfers as completed}
				<div class="rounded-lg border border-gray-300 px-4 py-2">
					<div class="flex w-full items-center gap-2">
						<p
							class="w-full min-w-[180px] overflow-hidden text-center overflow-ellipsis text-gray-600"
						>
							{completed.path.split(/\/|\\/).pop()}
						</p>
					</div>
					<p class="break mt-2 text-gray-400">
						{@html completed.path.replace(/(\\)|(\/)/g, '$1<wbr>')}
					</p>
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
					d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
				/>
			</svg>
		</div>
		<p class="text-lg text-gray-500">No hay transferencias finalizadas</p>
	{/if}
</div>
