import { readTextFile, BaseDirectory, readDir, exists, mkdir } from '@tauri-apps/plugin-fs';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
	// Check if the uploads directory exists
	const uploadsDirExists = await exists('uploads', {
		baseDir: BaseDirectory.AppLocalData,
	});
	if (!uploadsDirExists) {
		console.error('Uploads directory does not exist');

		await mkdir('uploads', {
			baseDir: BaseDirectory.AppLocalData,
		});
	}

	let recordings: string[] = [];
	try {
		const uploadsDir = await readDir('uploads', { baseDir: BaseDirectory.AppLocalData });
		recordings = uploadsDir.filter((e) => e.isDirectory).map((d) => d.name);
	} catch (err) {
		console.error('Error reading uploads directory:', err);
	}
	return { recordings };
};
