import { NodeConstants } from '../../constants'; 
/**
 * An ast node representing a single "let" statement
 */

export default class LetStatementNode {
    /**
     * @constructor
     * @param {string} target - the name of the targeted let expression.
     * @param {ExpressionNode | null} propAccess - dds property access to the target. Used to assign a value to reference type's member.
     * @param {ExpressionNode} evalutatedResult - the evaluted result to assign to the target
     */
    constructor(target, propAccess, evalutatedResult) {
        this.type = NodeConstants.LETSTATEMENT;
        this.target = target;
        this.propAccess = propAccess;
        this.evalutatedResult = evalutatedResult;
    }
}