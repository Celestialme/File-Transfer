<script lang="ts">
	import { config } from '$lib/store.svelte';
	import { login } from '$lib/utils';

	let isLoading = $state(false);
	let error = $state('');
	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		isLoading = true;
		await login({ username: config.username, password: config.password }).catch((e) => (error = e));
		isLoading = false;
	}
</script>

<div class="flex min-h-screen items-center justify-center bg-gray-100">
	<div class="w-96 rounded-lg bg-white p-8 shadow-md">
		<h2 class="mb-6 text-center text-2xl font-bold text-gray-800">Iniciar Sesión</h2>

		<form class="space-y-4" onsubmit={submit}>
			<div>
				<p class="block cursor-default text-sm font-medium text-gray-700">Usuario</p>
				<input
					type="text"
					bind:value={config.username}
					id="username"
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
					required
				/>
			</div>

			<div>
				<p class="block cursor-default text-sm font-medium text-gray-700">Contraseña</p>
				<input
					type="password"
					bind:value={config.password}
					id="password"
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
					required
				/>
			</div>

			<button
				type="submit"
				disabled={isLoading}
				class={`w-full rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:outline-none ${
					isLoading ? 'cursor-not-allowed opacity-50' : ''
				}`}
			>
				{isLoading ? 'Logging in...' : 'Login'}
			</button>

			<p class:invisible={!error} class=" min-h-[30px] text-center text-red-500">{error}</p>
		</form>
	</div>
</div>
