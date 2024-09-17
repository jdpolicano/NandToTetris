import { NodeConstants } from '../../constants'; 
/**
 * A AST node representing the subroutine body
 */
export default class SubroutineBodyNode {
    /**
     * @constructor
     * @param {Array[VarDecNode]} varDecs - an array of variable declarations
     * @param {
     * Array[IfStatementNode 
     * | LetStatementNode 
     * | WhileStatementNode 
     * | DoStatementNode 
     * | ReturnStatementNode]} statments - an array of AST node containting statements. 
     */
    constructor(varDecs, statments) {
        this.type = NodeConstants.SUBROUTINEBODY;
        this.varDecs = varDecs;
        this.statments = statments;
    }
}