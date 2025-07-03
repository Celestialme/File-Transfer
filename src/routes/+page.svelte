<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import Active from './components/Active.svelte';
	import Completed from './components/Completed.svelte';
	import Header from './components/Header.svelte';
	import Tabs from './components/Tabs.svelte';
	import { invoke } from '@tauri-apps/api';
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

<div
	class="flex min-h-screen w-full flex-col items-center bg-gradient-to-br from-sky-50 to-blue-100 p-4"
>
	<Header />

	<div class="flex items-center justify-between">
		<div>
			<h1 class="mb-2 text-4xl font-bold text-gray-800">Gestor de Transferencias</h1>
			<p class="text-gray-600">Transfiere archivos de forma segura y eficiente</p>
		</div>
	</div>
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
