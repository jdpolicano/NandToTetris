import fs from 'fs/promises';
import path from 'path';

/**
 * Will return every single file inside a source directroy/file matching the .jack extension 
 * @param {string} source - the file path candidate for compilation. 
 */
export async function getFilePaths(source) {
    const stats = await fs.stat(source);
    if (stats.isFile()) {
        return path.extname(source) === '.jack' ? [path.resolve(process.cwd(), source)] : [];
    } else if (stats.isDirectory()){
        const files = await fs.readdir(source);
        const fullPaths = [];

        for (const file of files) {
            const filePath = `${source}${path.sep}${file}`;
            const files = await getFilePaths(filePath);
            fullPaths.push(...files);
        }
        return fullPaths
    }

    return [];
}

