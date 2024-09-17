import { NodeConstants } from '../../constants'; 
/**
 * AST node representing a function parameter
 */
export default class ParameterNode {
    /**
     * @constructor
     * @param {string} dataType - data type for this parameter;
     * @param {string} name - name of this parameter;
     */
    constructor(dataType, name) {
        this.type = NodeConstants.PARAMETERLIST;
        this.dataType = dataType;
        this.name = name;
    }
}