/**
 * Class representing a tree node for our given tree configuration (either CST or AST)
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
}


