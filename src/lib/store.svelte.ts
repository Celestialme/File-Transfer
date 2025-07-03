import type { Config } from './types';
import { get_config } from './utils';

export let config = $state({
	username: '',
	password: '',
	folder_path: 'D:\\python\\codes\\fiverr\\testfolder\\f1',
	server_url: 'http://127.0.0.1:3000'
} as Config);
get_config().then((c) => {
	for (const [key, value] of Object.entries(c as Config)) {
		if (value !== null) {
			config[key as keyof Config] = value;
		}
	}
});
