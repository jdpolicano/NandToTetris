import { NodeConstants } from '../../constants'; 
import SubroutineDecNode from './SubroutineDecNode';
/**
 * An ast node representing a single "Do" statement
 */
export default class DoStatementNode {
    /**
     * @constructor
     * @param {SubroutineDecNode} subroutine - the subroutine to call.
     */
    constructor(subroutine) {
        this.type = NodeConstants.DOSTATEMENT;
        this.subroutine = subroutine;
    }
}