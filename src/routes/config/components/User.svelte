<script>
	import { config } from '$lib/store.svelte';
	import { logout } from '$lib/utils';
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<span class="text-sm font-medium text-gray-700">User Name</span>
		<span
			class="text-md w-[100px] rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-center text-gray-600"
		>
			{config.username}
		</span>
	</div>
	<div class="w-full text-right">
		<button
			onclick={logout}
			class="w-[100px] cursor-pointer rounded-md bg-blue-600 px-4 py-2 text-white transition-colors duration-200 hover:bg-blue-700"
			>logout</button
		>
	</div>
</div>
