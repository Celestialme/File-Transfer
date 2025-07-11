import { invoke } from '@tauri-apps/api';
import { config } from './store.svelte';

export async function update_config() {
	await invoke('update_config', { config, restart: true });
}
export async function get_config() {
	return await invoke('get_config');
}

export async function login({ username, password }: { username: string; password: string }) {
	return await invoke('login', { username, password });
}
export async function logout() {
	return await invoke('logout');
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

export async function open_folder() {
	return await invoke('open_folder', { path: config.folder_path });
}
export async function force_sync() {
	return await invoke('force_sync');
}
