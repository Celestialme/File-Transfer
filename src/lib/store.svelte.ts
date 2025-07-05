import type { Config } from './types';
import { get_config } from './utils';

export let config = $state({} as Config);
get_config().then((c) => {
	for (const [key, value] of Object.entries(c as Config)) {
		if (value !== null) {
			config[key as keyof Config] = value;
		}
	}
});
