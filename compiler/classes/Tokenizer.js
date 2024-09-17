import Token from "./primitives/Token.js";
import { TokenConstants, Keywords, Symbols } from "../constants.js";
/**
 * Main class responsible for reading a source file and breaking it into valid tokens. 
 * Skips newline characters (\n, \n\r, and \r are all considered newlines) 
 */
export default class Tokenizer {
    #charGenerator = null;
    /**
     * @constructor
     * @param {string} source - the file string to tokenize
     * @param {string} filePath - the path the the file (useful for error messages) 
     */
    constructor(source, filePath) {
        this.filePath = filePath;
        this.lineCount = 0;
        this.charCount = 0;
        this.file = source;
        this.fileIndex = 0;
        this.defaultCharacterTabCount = 4; // 4 spaces for one tab generally.
        this.done = false; 
    }

    /**
     * Returns the next valid token from the file or null if none is present.
     * @returns {null | Token}
     */
    getNextToken() {
        // Position stream at the next valid character
        this.jumpToNextValidCharacter();
        // Ensure we haven't hit EOF
        if (this.done) return null;

        // Read in the next char
        let currentToken = this.getChar();


        // Process and return the appropriate token
        if (this.isSymbolConstant(currentToken)) {
            return this.processSymbolConstant(currentToken);
        } else if (currentToken === '"') {
            return this.processStringConstant(currentToken);
        } else {
            return this.processKeywordIdentifierOrIntegerConstant(currentToken);
        }
    }

    /**
     * Allows you to peek at the next n tokens, and then reverts the file stream back to its original state;
     * @param {integer} depth - the depth of tokens to check out
     */
    peekToken(depth = 1) {
        const fileIndex = this.fileIndex;
        const lineCount = this.lineCount;
        const charCount = this.charCount; 
        const done = this.done;
        const result = Array(depth).fill(null).map(_ => this.getNextToken());

        this.fileIndex = fileIndex;
        this.lineCount = lineCount;
        this.charCount = charCount;
        this.done = done; 

        return result; 
    }

    /**
     * Processes a symbol constant and returns a new token.
     * @param {string} currentToken - The current token being processed
     * @returns {Token}
     */
    processSymbolConstant(currentToken) {
        return this.createToken(TokenConstants.SYMBOL, currentToken);
    }

    /**
     * Processes a string constant and returns a new token.
     * @param {string} currentToken - The current token being processed
     * @returns {Token}
     */
    processStringConstant(currentToken) {
        currentToken += this.getChar();
        while (currentToken[currentToken.length - 1] !== '"' && !this.done) {
            currentToken += this.getChar();
        }

        if (!this.isStringConstant(currentToken)) {
            this.throwTokenError('String Constant');
        }

        return this.createToken(TokenConstants.STRING, currentToken);
    }

    /**
     * Processes a keyword, identifier, or integer constant and returns a new token.
     * @param {string} currentToken - The current token being processed
     * @returns {Token}
     */
    processKeywordIdentifierOrIntegerConstant(currentToken) {
        while (!this.isSymbolConstant(this.peek()) && this.isValidCharacter()) {
            currentToken += this.getChar();
        }

        if (this.isKeyword(currentToken)) {
            return this.createToken(TokenConstants.KEYWORD, currentToken);
        } else if (this.isIntegerConstant(currentToken)) {
            return this.createToken(TokenConstants.INTEGER, currentToken);
        } else if (this.isIdentifier(currentToken)) {
            return this.createToken(TokenConstants.IDENTIFIER, currentToken);
        } else {
            this.throwTokenError('Unknown token');
        }
    }

    /**
     * Jumps to the next valid character to process
     */
    jumpToNextValidCharacter() {
        if (this.isSpace()) {
            this.skipSpace();
            return this.jumpToNextValidCharacter();
        }

        if (this.isNewline()) {
            this.skipNewlines();
            return this.jumpToNextValidCharacter();
        }

        if (this.isSingleComment()) {
            this.skipSingleComment();
            return this.jumpToNextValidCharacter();
        }

        if (this.isMultilineComment()) {
            this.skipMultilineComment();
            return this.jumpToNextValidCharacter();
        }

        if (this.isCharacterTabulation()) {
            this.skipCharacterTabulation();
            return this.jumpToNextValidCharacter();
        }
    }
    /**
     * Skips to a given point in the stream where a certain character occurs next. 
     * @param {string} match - the character(s) that we want to match and jump to.
     */
    jumpTo(match) {
        while (this.peek(this.fileIndex, this.fileIndex + match.length) !== match && !this.done) {
            this.getChar();
        } 
    }

    /**
     * Skips spaces
     */
    skipSpace() {
        while(this.isSpace()) this.getChar();
    }

    /**
     * Skips newlines
     */
    skipNewlines() {
        while (this.isNewline() && !this.done) {
            const char = this.getChar();
            if (char === '\n') {
                this.lineCount++;
            }
        } 
        this.charCount = 0;
    }

    /**
     * Skips single comment
     */
    skipSingleComment() {
        this.jumpTo('\n');
        this.skipNewlines();
    }

    /**
     * Skips a multiline comment
     */
    skipMultilineComment() {
        this.jumpTo('*/');
        // skip *
        this.getChar();
        // skip /
        this.getChar();
    }

    /**
     * Skips unicode value 09 (character tabulation);
     */

    skipCharacterTabulation() {
        while (this.isCharacterTabulation() && !this.done) {
            const charCount = this.charCount + this.defaultCharacterTabCount;
            this.getChar();
            this.charCount = charCount;
        }
    }

    /**
     * Determines if current character is a comment; 
     * @returns {boolean}
     */
    isSingleComment() {
        const comments = this.peek(this.fileIndex, this.fileIndex + 2);
        return comments === '//';
    }

    /**
     * Determines if current character is a comment; 
     * @returns {boolean}
     */
    isMultilineComment() {
        return this.peek(this.fileIndex, this.fileIndex + 3) === '/**' || this.peek(this.fileIndex, this.fileIndex + 2) === '/*';
    }

    isMultilineClosingTag() {
        return this.peek(this.fileIndex, this.fileIndex + 2) === '*' && this.peek(this.fileIndex + 1) === '/'
    }

    /**
     * Determines if current character is a newline
     * @returns {boolean}
     */
    isNewline() {
        const char = this.peek();
        return char === '\n' || char === '\r';
    }

    /**
     * Determines if current character is a space
     * @returns {boolean}
     */
    isSpace() {
        return this.peek() === ' '; 
    }

    /**
     * Determines if the current character is character tabulation
     */
    isCharacterTabulation() {
        return this.peek() === '\t';
    }

    /**
     * Determines if a given position is valid
     * @returns {boolean}
     */
    isValidCharacter() {
        return !(this.isSpace() || this.isNewline() || this.isSingleComment() || this.isMultilineComment() || this.done);
    }
    /**
     * Determines if the current character is a Symbol
     * @returns {boolean}
     */
    isSymbolConstant(word) {
        return Symbols.hasOwnProperty(word);
    }

    /**
     * Determines if a word is a keyword or not
     * @param {string} word - the piece of code to determine if its a keyword
     * @returns {boolean}
     */
    isKeyword(word) {
        return Keywords.hasOwnProperty(word);
    }

    /**
     * Determines if a word is an a valid integer constant or not 
     * @param {string} word - the piece of code to determine if its a keyword
     * @returns {boolean | error}
     */
    isIntegerConstant(word) {
        if (!isNaN(word[0])) {
            for (const char of word) {
                if (isNaN(char)) {
                    this.throwTokenError('Integer Constant');
                }
            }
            return true; 
        }
        return false;
    }

    /**
     * Determines if if a word is a string constant
     * @param {string} word - the phrase to consider
     * @returns {boolean}
     */
    isStringConstant(word) {
        if (word[0] === '"') {
            if (word.length < 2 || word[word.length - 1] !== '"') {
                this.throwTokenError('String Constant');
            }
            return true;
        }
        return false;
    }
    /**
     * Determines if if a word is an identifier. Any series of chas, digits, _ not starting with a digit. Last check in the sequence so 
     * should throw if it cant recognize it. 
     * @param {string} word - the phrase to consider
     * @returns {boolean}
     */
    isIdentifier(word) {
        const regex = /^[a-zA-Z_][a-zA-Z_0-9]*$/; // starts with a alnum seq and doesn't have any special chars;
        
        if (!regex.test(word)) {
            this.throwTokenError('Identifier');
        }

        return true;
    }

    /**
     * Lets you look foward in the character stream without advancing the pointer
     * @param {integer} from [this.fileIndex] - the index you want to see from. If to is undefined will return single char.
     * @param {integer} to [this.fileIndex + 1] - the index you want to see from. If to is undefined will return single char.
     * @returns {string};
     */
    peek(from = this.fileIndex, to = from + 1) {
        if (from < this.file.length) {
            to = Math.min(to, this.file.length);
            return this.file.slice(from, to) || null;
        }
        return null;
    }

    /**
     * @returns Character generator that slowly advances the fileIndex
     */
    * #getCharacterGenerator() {
        while (true) {
            if (this.fileIndex < this.file.length) {
                const char = this.file[this.fileIndex];
                this.fileIndex++;
                this.charCount++;
                this.done = this.fileIndex >= this.file.length; 
                yield char;
                continue;
            }
            yield null
        }
    }

    /**
     * Checks if we've created a generator then returns the next char in the stream. 
     */
    getChar() {
        if (this.#charGenerator === null) {
            this.#charGenerator = this.#getCharacterGenerator();
        }
        const { value } = this.#charGenerator.next();
        return value; 
    }


    /**
     * Creates a new Token from a word and type
     * @param {string} type - the type of token
     * @param {string} raw - the data in the token
     */
    createToken(type, raw) {
        return new Token(type, raw, this.lineCount + 1, this.charCount);
    }


    /**
     * Throws error message to the silly dev.
     * @param {string} operation - the op that caused the error
     */
    throwTokenError(operation) {
        const err = `Error Tokenizing ${operation} in ${this.filePath} @ ${this.lineCount}:${this.charCount}`
        throw new Error(err);
    }

}