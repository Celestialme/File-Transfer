<script lang="ts">
	import User from './components/User.svelte';
	import Folders from './components/Folders.svelte';
	import Connection from './components/Connection.svelte';
	import Sync from './components/Sync.svelte';
	const tabs = $state([
		{ label: '🔐 Usuario y autenticación', component: User },
		{ label: '📁 Carpetas', component: Folders },
		{ label: '🌐 Conexión', component: Connection },
		{ label: '🔄 Sincronización', component: Sync }
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
						<svg
							class="h-5 w-5 text-gray-600"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width={2}
								d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
							/>
						</svg>
					</div>
					<h1 class="text-xl font-semibold text-gray-900">Configuración</h1>
				</div>
			</div>
		</div>

		<div class="border-b border-gray-200">
			<nav class="flex space-x-0">
				{#each tabs as tab}
					{@render Tab({
						label: tab.label,
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
	isActive,
	label,
	onclick = () => {}
}: {
	label: string;
	isActive: boolean;
	onclick?: () => void;
})}
	<button
		{onclick}
		class={`border-b-2 px-6 py-3 text-sm font-medium transition-colors duration-200 ${
			isActive
				? 'border-blue-500 bg-blue-50 text-blue-600'
				: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
		}`}
	>
		{label}
	</button>
{/snippet}
