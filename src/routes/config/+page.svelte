<script lang="ts">
	import User from './components/User.svelte';
	import Folder from './components/Folder.svelte';
	import Connection from './components/Connection.svelte';
	import {
		User as UserIcon,
		Folder as FolderIcon,
		Cable as ConnectionIcon,
		Cog
	} from '@lucide/svelte';
	const tabs = $state([
		{ label: 'User Authentication', component: User, icon: UserIcon },
		{ label: 'Folders', component: Folder, icon: FolderIcon },
		{ label: 'Connection', component: Connection, icon: ConnectionIcon }
	]);

	let activeTab = $state(tabs[0]);
</script>

<div class="min-h-screen bg-gray-50 p-8">
	<div class="mx-auto max-w-4xl">
		<div class="rounded-lg border border-gray-200 bg-white shadow-sm">
			<!-- Header -->
			<div class="border-b border-gray-200 px-6 py-4">
				<div class="flex items-center space-x-3">
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-gray-100">
						<Cog></Cog>
					</div>
					<h1 class="text-xl font-semibold text-gray-900">Configuration</h1>
				</div>
			</div>
		</div>

		<div class="border-b border-gray-200">
			<nav class="flex w-full space-x-0">
				{#each tabs as tab}
					{@render Tab({
						tab,
						isActive: tab == activeTab,
						onclick: () => (activeTab = tab)
					})}
				{/each}
			</nav>
		</div>
		<div class="p-6">
			<activeTab.component />
		</div>
	</div>
</div>

{#snippet Tab({
	tab,
	isActive,
	onclick = () => {}
}: {
	tab: {
		label: string;
		component: any;
		icon: any;
	};
	isActive: boolean;
	onclick?: () => void;
})}
	<button
		{onclick}
		class={`w-full border-b-2 px-6 py-3 text-sm font-medium transition-colors duration-200 ${
			isActive
				? 'border-blue-500 bg-blue-50 text-blue-600'
				: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
		}`}
	>
		<div class="flex w-full items-center justify-center gap-2">
			<tab.icon />
			{tab.label}
		</div>
	</button>
{/snippet}
