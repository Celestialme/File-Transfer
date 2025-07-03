import { invoke } from '@tauri-apps/api';
import { config } from './store.svelte';

export async function update_config() {
	await invoke('update_config', { config });
}
export async function get_config() {
	return await invoke('get_config');
}
