<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import Active from './components/Active.svelte';
	import Completed from './components/Completed.svelte';
	import Header from './components/Header.svelte';
	import Tabs from './components/Tabs.svelte';
	import { invoke } from '@tauri-apps/api';
	import { isConnected } from '$lib/store.svelte';
	import { Loader } from '@lucide/svelte';
	type Transfer = {
		path: string;
		progress: number;
		state: 'active' | 'completed';
		type: 'download' | 'upload';
	};
	listen('transfer', (event) => {
		let data = event.payload as Transfer;
		console.log(data);
		if (data.state === 'completed') {
			completedTransfers[data.path] = data;
			delete activeTransfers[data.path];
		} else {
			activeTransfers[data.path] = data;
		}
	});
	listen('is_connected', (event) => {
		console.log(event);
		isConnected.set(event.payload as boolean);
		console.log(isConnected);
	});
	invoke('get_completed_transfers').then((data) => {
		completedTransfers = (data as Transfer[]).reduce(
			(acc, transfer) => {
				acc[transfer.path] = transfer;
				return acc;
			},
			{} as { [key: string]: Transfer }
		);
	});
	let activeTab = $state('active');
	let activeTransfers: { [key: string]: Transfer } = $state({});
	let completedTransfers: { [key: string]: Transfer } = $state({});
	let activeTransfersArray = $derived(Object.values(activeTransfers));
	let completedTransfersArray = $derived(Object.values(completedTransfers));
</script>

{#if !isConnected.get()}
	<div
		class=" fixed top-[50px] left-0 z-50 flex h-screen w-screen flex-col items-center justify-center bg-gray-800"
	>
		<Loader class="mb-6 h-16 w-16 animate-spin text-white" />
		<p class="mb-2 text-xl font-semibold text-white">Connection Lost</p>
		<p class="text-md text-white">Changes will not be saved</p>
	</div>
{/if}

<div
	class="flex min-h-screen w-full flex-col items-center bg-gradient-to-br from-sky-50 to-blue-100 p-4"
>
	<Header />
	<Tabs
		bind:activeTab
		activeTransfers={activeTransfersArray}
		completedTransfers={completedTransfersArray}
	/>
	{#if activeTab === 'active'}
		<Active transfers={activeTransfersArray} />
	{:else if activeTab === 'completed'}
		<Completed transfers={completedTransfersArray} />
	{/if}
</div>
