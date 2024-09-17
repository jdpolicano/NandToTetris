import { NodeConstants } from '../../constants'; 

/**
 * AST node for a classVarDec.
 */
export default class ClassVarDecNode {
    /**
     * @constructor
     * @param {string} varType - type of classvar, either field or static
     * @param {string} dataType - the type of of this var (int, bool, etc...)
     * @param {string} varName - the name of this var.
     */
    constructor(varType, dataType, name) {
        this.type = NodeConstants.CLASSVARDEC;
        this.varType = varType;
        this.dataType = dataType;
        this.name = name;
    }
}