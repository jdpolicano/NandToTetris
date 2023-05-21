import { NodeConstants } from '../../constants'; 
/**
 * An ast node representing a single "if" statement
 */

export default class IfStatementNode {
    /**
     * @constructor
     * @param {ExpressionNode} condition - the condition to evaluate.
     * @param {
     * Array[IfStatementNode 
     * | LetStatementNode 
     * | WhileStatementNode 
     * | DoStatementNode 
     * | ReturnStatementNode]} statements - the statements to evalutate on true
     * @param {
     * Array[IfStatementNode 
     * | LetStatementNode 
     * | WhileStatementNode 
     * | DoStatementNode 
     * | ReturnStatementNode]} _else - the statements to evalute if false. Optional. 
     */
    constructor(condition, statements, _else) {
        this.type = NodeConstants.IFSTATEMENT;
        this.condition = condition;
        this.statements = statements;
        this._else = _else;
    }
}