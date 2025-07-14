<script lang="ts">
	import type { Transfer } from '$lib/types';
	import Progress from './Progress.svelte';
	import { Download } from '@lucide/svelte';
	let {
		transfers
	}: {
		transfers: Transfer[];
	} = $props();
</script>

<div class="flex h-[320px] max-h-[320px] w-full flex-col overflow-auto py-12 text-center">
	{#if transfers.length > 0}
		<div class="flex flex-col gap-2">
			{#each transfers as active}
				<div
					class="rounded-lg border border-l-[5px] border-gray-300 px-4 py-2"
					class:upload={active.type === 'upload'}
					class:download={active.type === 'download'}
				>
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
		<Download class="mx-auto my-auto h-16 w-16 text-gray-400" />
		<p class="text-lg text-gray-500">There are no transfers in progress</p>
	{/if}
</div>

<style>
	.upload {
		border-left-width: 5px;
		border-left-color: red;
	}
	.download {
		border-left-width: 5px;
		border-left-color: green;
	}
</style>
