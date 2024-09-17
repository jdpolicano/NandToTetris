import { NodeConstants } from '../../constants'; 
/**
 * AST node for a subroutine - function, method, or constructor
 */
export default class SubroutineDecNode {
    /**
     * @constructor
     * @param {string} nodeType - type of node either constructor, method, or function
     * @param {string} returnType - one of the primitive types that the sub is expected to return
     * @param {string} name - the name of the sub
     * @param {Array[ParameterNode]} parameters - the list of parameters
     * @param {SubroutineBodyNode} body - the body of the sub.
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