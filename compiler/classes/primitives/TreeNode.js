/**
 * Class representing a CST tree node
 */
export default class TreeNode {
    constructor(type, value = null) {
        this.type = type;
        this.value = value;
        this.children = [];
    }

    addChild(child) {
        this.children.push(child);
    }

    addMetaData(fileName, token) {
        this._metaData = {
            fileName: fileName,
            lineCount: token.lineCount,
            charCount: token.charCount
        }
    }
}


