export type Transfer = {
	path: string;
	progress: number;
	state: 'active' | 'completed';
	type: 'download' | 'upload';
};

export type Config = {
	username: string;
	password: string;
	server_url: string;
	folder_path: string;
	token: string;
	refresh_token: string;
};
