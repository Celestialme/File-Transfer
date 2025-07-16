import type { Config } from './types';
import { get_config } from './utils';
import { invoke } from '@tauri-apps/api';

export let config = $state({} as Config);
let is_connected = $state(true);

export let isConnected = {
	get: () => is_connected,
	set: (value: boolean) => (is_connected = value)
};

invoke('check_connection').then((value) => (is_connected = value as boolean));

get_config().then((c) => {
	for (const [key, value] of Object.entries(c as Config)) {
		if (value !== null) {
			config[key as keyof Config] = value;
		}
	}
});
