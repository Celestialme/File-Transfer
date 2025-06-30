<script>
	let credentials = {
		username: 'user',
		password: '••••••••••'
	};
	let isEditingPassword = false;
	let tempPassword = '';

	function handlePasswordEdit() {
		isEditingPassword = !isEditingPassword;
		if (!isEditingPassword) {
			credentials.password = tempPassword;
			tempPassword = '';
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<span class="text-sm font-medium text-gray-700">Nombre de usuario</span>
		<span class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-600">
			{credentials.username}
		</span>
	</div>

	<div class="flex items-center justify-between">
		<span class="text-sm font-medium text-gray-700">Contraseña</span>
		<div class="flex items-center space-x-3">
			{#if isEditingPassword}
				<input
					type="password"
					value={tempPassword}
					class="w-64 rounded-md border border-gray-300 px-3 py-2 focus:border-transparent focus:ring-2 focus:ring-blue-500 focus:outline-none"
					placeholder="Nueva contraseña"
				/>
			{:else}
				<span
					class="w-64 rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-600"
				>
					{credentials.password}
				</span>
			{/if}
			<button
				onclick={handlePasswordEdit}
				class="rounded-md bg-blue-600 px-4 py-2 text-white transition-colors duration-200 hover:bg-blue-700"
			>
				{isEditingPassword ? 'Guardar' : 'Modificar'}
			</button>
		</div>
	</div>
</div>
