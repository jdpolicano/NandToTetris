import { NodeConstants } from '../../constants'; 
/**
 * An ast node representing a single "While" statement
 */

export default class WhileStatementNode {
    /**
     * @constructor
     * @param {ExpressionNode} condition - the condition to evaluate.
     * @param {
     * Array[IfStatementNode 
     * | LetStatementNode 
     * | WhileStatementNode 
     * | DoStatementNode 
     * | ReturnStatementNode]} statements - the statements to evalutate on true
     */
    constructor(condition, statements, _else) {
        this.type = NodeConstants.WHILESTATEMENT;
        this.condition = condition;
        this.statements = statements;
    }
}