import { invoke } from '@tauri-apps/api';
import { config } from './store.svelte';

export async function update_config() {
	await invoke('update_config', { config });
}
export async function get_config() {
	return await invoke('get_config');
}

export async function login({ username, password }: { username: string; password: string }) {
	return await invoke('login', { username, password });
}
export async function save_initial_config({
	serverUrl,
	folderPath
}: {
	serverUrl: string;
	folderPath: string;
}) {
	return await invoke('save_initial_config', { serverUrl, folderPath });
}
