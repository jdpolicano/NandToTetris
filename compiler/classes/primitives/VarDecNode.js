import { NodeConstants } from '../../constants'; 
/**
 * An ast node representing a variable declaration
 */

export default class VarDecNode {
    constructor(dataType, name) {
        this.type = NodeConstants.VARDEC;
        this.dataType = dataType;
        this.name = name; 
    }
}