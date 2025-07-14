<script>
	let { activeTab = $bindable(), activeTransfers, completedTransfers } = $props();
</script>

<div class="my-6">
	<div class="border-b border-gray-200">
		<nav class="-mb-px flex space-x-8">
			<button
				onclick={() => (activeTab = 'active')}
				class={`cursor-pointer border-b-2 px-1 py-2 text-sm font-medium ${
					activeTab === 'active'
						? 'border-blue-500 text-blue-600'
						: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
				}`}
			>
				Current transfers ({activeTransfers.length})
			</button>
			<button
				onclick={() => (activeTab = 'completed')}
				class={`cursor-pointer border-b-2 px-1 py-2 text-sm font-medium ${
					activeTab === 'completed'
						? 'border-blue-500 text-blue-600'
						: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
				}`}
			>
				Completed transfers ({completedTransfers.length})
			</button>
		</nav>
	</div>
</div>
