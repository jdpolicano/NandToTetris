/**
 * Class representing a token. Can be of type Keyword, Symbol, IntegerConstant, StringConstant, or Identifier
 */
export default class Token {
    /**
     * 
     * @param {TokenConstant} type - type of token
     * @param {string} raw - raw data 
     * @param {integer} lineCount - line number it was encountered on
     */
    constructor(type, raw, lineCount, charCount) {
        this.type = type;
        this.raw = raw; 
        this.lineCount = lineCount;
        this.charCount = charCount;
    }
}