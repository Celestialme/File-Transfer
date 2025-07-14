<script lang="ts">
	import type { Transfer } from '$lib/types';
	import { CircleCheck } from '@lucide/svelte';

	let {
		transfers
	}: {
		transfers: Transfer[];
	} = $props();
</script>

<div class="flex h-[320px] max-h-[320px] w-full flex-col overflow-auto py-12 text-center">
	{#if transfers.length > 0}
		<div class="flex flex-col gap-2">
			{#each transfers as completed}
				<div
					class:upload={completed.type === 'upload'}
					class:download={completed.type === 'download'}
					class="rounded-lg border border-l-[5px] border-gray-300 px-4 py-2"
				>
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
		<CircleCheck class="mx-auto my-auto mb-4 h-16 w-16  text-gray-400" />
		<p class="mt-auto text-lg text-gray-500">There are no completed transfers</p>
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
