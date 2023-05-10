import { expect } from 'expect';

export default class Cli {
    /**
     * @constructor
     * @param {string} programName - the name of the program
     * @param {string} author - the name of this prgorams master
     */
    constructor(programName, author) {
        this.programName = programName;
        this.author = author
        this.commands = [];
        this.flags = new Map();
    }

    /**
     * Adds a new command line argument to the program.
     * @param {Object} command - the meta data on the command line argument
     * @param {string} command.prefix - the character prefix each argument should start with.
     * @param {string} command.short - the abbreviated version of the command.
     * @param {string} command.long - the full name of the command line argument.
     * @param {string} command.description - the full description of usage.
     * @param {*} command.default - the default value for this command
     */
    addCommand(command) {
        this.validateCommand(command);
        this.commands.push(command); 
        this.flags.set(command.long, command.default);
    }

    /**
     * Validates the form of the command object passed to add Command
     * @param {Object} command - the meta data on the command line argument
     * @param {string} command.prefix - the character prefix each argument should start with.
     * @param {string} command.short - the abbreviated version of the command.
     * @param {string} command.long - the full name of the command line argument.
     * @param {string} command.description - the full description of usage. 
     * @param {*} command.default - the default value for this command
     */
    validateCommand(command) {
        expect(command).toEqual(expect.any(Object))
        expect(command.prefix).toEqual(expect.any(String));
        expect(command.short).toEqual(expect.any(String));
        expect(command.long).toEqual(expect.any(String));
        expect(command.description).toEqual(expect.any(String));
        expect(command.default).not.toBeUndefined();
    }

    /**
     * Parsesc arguments and adds them to our map of flags passed by the user;
     */
    parseArgs() {
        const argv = process.argv.slice(2);
        
        if (argv.length % 2) throw new Error(`Invalid number of args, expected ${args.length + 1} received ${args.length}`);

        for (let i = 0; i < argv.length; i += 2) {
            const flag = argv[i];
            const value = argv[i + 1];
            let matched = false;

            for (let j = 0; j < this.commands.length; j++) {
                const { short, long, prefix } = this.commands[j];
                if (flag === `${prefix}${short}` || flag === `${prefix}${long}`) {
        
                    if (value[0] !== prefix) {
                        this.flags.set(long, value);
                        matched = true;
                    } else {
                        throw new Error(`Invalid argument ${flag} prefix must not be included in parsed value`);
                    }
                };
            }

            if (!matched) {
                throw new Error(`No argument matched for ${flag}`);
            }
        }
        return;
    }

    /**
     * Runs the program - basic idea is a user can overwrite this and build their own cli this way. Very simple :). 
     */
    run() {
        console.log('Overwrite me :)');
    }
}