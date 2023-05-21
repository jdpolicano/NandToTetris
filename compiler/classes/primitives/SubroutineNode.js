import { NodeConstants } from '../../constants'; 
/**
 * AST node for a subroutine - function, method, or constructor
 */
export default class SubroutineNode {
    /**
     * @constructor
     * @param {string} nodeType - type of node either constructor, method, or function
     * @param {string} returnType - one of the primitive types that the sub is expected to return
     * @param {string} name - the name of the sub
     * @param {string} parameters - the list of parameters
     * @param {Array} body - the body of the sub.
     */
    constructor(routineType, returnType, name, parameters, body) {
        this.type = NodeConstants.SUBROUTINEDEC;
        this.routineType = routineType;
        this.returnType = returnType;
        this.name = name;
        this.parameters = parameters;
        this.body = body;
    }
}