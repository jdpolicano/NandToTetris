import Cli from './classes/Cli.js'; 
import Tokenizer from './classes/Tokenizer.js';
import CSTParser from './classes/Parser.js';
import path from 'path'; 
import fs from 'fs';
import { getFilePaths } from './fileHelpers.js';


async function run() {
    const start = Date.now();
    const cli = new Cli('Jack Compiler', 'Jake Policano');

    cli.addCommand({
        prefix: '-',
        short: 's',
        long: 'source',
        description: 'The source file/directory to compile',
        default: process.cwd()
    });
    
    cli.addCommand({
        prefix: '-',
        short: 'o',
        long: 'output',
        description: 'the directory to output compiled files to',
        default: process.cwd()
    });
    
    cli.parseArgs();

    const source = path.resolve(process.cwd(), cli.flags.get('source'));
    const output = path.resolve(process.cwd(), cli.flags.get('output'));
    const inputFilePaths = await getFilePaths(source);

    if (inputFilePaths.length) {
        const outputFileStream = fs.createWriteStream(path.resolve(output, 'out.txt'));
        
        for (const filePath of inputFilePaths) {
            outputFileStream.write(filePath + '\n');
            const file = fs.readFileSync(filePath, { encoding: 'utf8' });
            const tokenizer = new Tokenizer(file, filePath);
            const parser = new CSTParser(tokenizer);
            const tree = parser.parseClass();
            outputFileStream.write(JSON.stringify(tree, null, 2) + '\n');

        }
        outputFileStream.close();
    }

    const runtime = Date.now() - start;
    const fileCount = inputFilePaths.length;
    console.log(`runtime: ${runtime}ms`)
    console.log(`parsed ${fileCount} files`);
    console.log(`average: ${(runtime / fileCount).toFixed(3)}ms/file`);
}

run();

