import { NodeConstants } from '../../constants'; 
/**
 * AST node representing a class. 
 */
export default class ClassNode {
    /**
     * @constructor
     * @param {string} name - the name of the class
     * @param {Array[Nodes]} classVars - array of classVar nodes
     * @param {Array[Nodes]} subroutines - array of subroutine nodes
     */
    constructor(name, classVars, subroutines) {
        this.type = NodeConstants.CLASS;
        this.name = name;
        this.classVars = classVars;
        this.subroutines = subroutines;
    }
}