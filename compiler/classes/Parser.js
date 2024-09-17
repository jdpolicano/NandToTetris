import TreeNode from "./primitives/TreeNode.js";
import { TokenConstants, NodeConstants } from '../constants.js'; 

export default class CSTParser {
    /**
     * @construtor
     * @param {Tokenizer} tokenizer - the given tokenizer for our files;
     */
    constructor(tokenizer) {
        this.tokenizer = tokenizer;
        this.lookahead = tokenizer.getNextToken();
    };

    // Parses a class declartion and adds it to our tree; 
    parseClass() {
        const classNode = new TreeNode(NodeConstants.CLASS);
        classNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        classNode.addChild(this.matchKeyword(NodeConstants.CLASS));
        classNode.addChild(this.parseClassName());
        classNode.addChild(this.matchSymbol('{'));
        while (this.isClassVarDec()) {
            classNode.addChild(this.parseClassVarDec());
        }
        while (this.isSubroutineDec()) {
            classNode.addChild(this.parseSubroutineDec());
        }
        classNode.addChild(this.matchSymbol('}'));
    
        return classNode;
    };
    

    parseClassName() {
        return this.matchIdentifier();
    };

    parseVarName() {
        return this.matchIdentifier();
    };

    parseSubroutineName() {
        return this.matchIdentifier();
    };

    parseType() {
        const typeNode = new TreeNode(NodeConstants.TYPE);
        typeNode.addMetaData(this.tokenizer.filePath, this.lookahead);

        if (this.lookahead.type === TokenConstants.IDENTIFIER) {
            typeNode.addChild(this.matchIdentifier());
            return typeNode;
        } else {
            typeNode.addChild(this.matchKeywordMultiple(['int', 'char', 'boolean']));
            return typeNode; 
        }
    };

    parseClassVarDec() {
        const classVarNode = new TreeNode(NodeConstants.CLASSVARDEC);
        classVarNode.addMetaData(this.tokenizer.filePath, this.lookahead);

        classVarNode.addChild(this.matchKeywordMultiple(['static', 'field']));
        classVarNode.addChild(this.parseType());
        classVarNode.addChild(this.parseVarName());
        while (this.isList()) {
            classVarNode.addChild(this.matchSymbol(','));
            classVarNode.addChild(this.parseVarName()); 
        }

        classVarNode.addChild(this.matchSymbol(';'));
        return classVarNode;
    };

    parseSubroutineDec() {
        const subroutineDecNode = new TreeNode(NodeConstants.SUBROUTINEDEC);
        subroutineDecNode.addMetaData(this.tokenizer.filePath, this.lookahead);

        subroutineDecNode.addChild(this.matchKeywordMultiple(['constructor', 'function', 'method']));
        if (this.isVoid()) {
            subroutineDecNode.addChild(this.matchKeyword('void'));
        } else {
            subroutineDecNode.addChild(this.parseType());
        };

        subroutineDecNode.addChild(this.parseSubroutineName());
        subroutineDecNode.addChild(this.matchSymbol('('));
        subroutineDecNode.addChild(this.parseParameterList());
        subroutineDecNode.addChild(this.matchSymbol(')'));
        subroutineDecNode.addChild(this.parseSubroutineBody());

        return subroutineDecNode; 
    };

    parseSubroutineBody() {
        const subroutineBodyNode = new TreeNode(NodeConstants.SUBROUTINEBODY);
        subroutineBodyNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        subroutineBodyNode.addChild(this.matchSymbol('{'));

        while (this.isVarDec()) {
            subroutineBodyNode.addChild(this.parseVarDec());
        } 
        subroutineBodyNode.addChild(this.parseStatements());
        subroutineBodyNode.addChild(this.matchSymbol('}'));
        return subroutineBodyNode;
    };

    parseVarDec() {
        const varDecNode = new TreeNode(NodeConstants.VARDEC);
        varDecNode.addMetaData(this.tokenizer.filePath, this.lookahead);

        varDecNode.addChild(this.matchKeyword('var'));
        varDecNode.addChild(this.parseType());
        varDecNode.addChild(this.parseVarName());

        while (this.isList()) {
            varDecNode.addChild(this.matchSymbol(','));
            varDecNode.addChild(this.parseVarName());
        }

        varDecNode.addChild(this.matchSymbol(';'));
        return varDecNode;
    };

    parseParameterList() {
        const parameterListNode = new TreeNode(NodeConstants.PARAMETERLIST);
        parameterListNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        
        if (this.isGroupClosingTag()) {
            return parameterListNode;
        }

        parameterListNode.addChild(this.parseType());
        parameterListNode.addChild(this.parseVarName());

        while (this.isList()) {
            parameterListNode.addChild(this.matchSymbol(','))
            parameterListNode.addChild(this.parseType());
            parameterListNode.addChild(this.parseVarName()); 
        };
        return parameterListNode;
    };

    parseStatements() {
        const statementsNode = new TreeNode(NodeConstants.STATEMENTS);
        statementsNode.addMetaData(this.tokenizer.filePath, this.lookahead);

        while (this.isStatement()) {
            if (this.lookahead.raw === 'let') {
                statementsNode.addChild(this.parseLetStatement());
            } else if (this.lookahead.raw === 'if') {
                statementsNode.addChild(this.parseIfStatement());
            } else if (this.lookahead.raw === 'while') {
                statementsNode.addChild(this.parseWhileStatement());
            } else if (this.lookahead.raw === 'do') {
                statementsNode.addChild(this.parseDoStatement());
            } else if (this.lookahead.raw === 'return') {
                statementsNode.addChild(this.parseReturnStatement());
            };
        };

        return statementsNode;
    };

    parseLetStatement() {
        const letStatementNode = new TreeNode(NodeConstants.LETSTATEMENT);
        letStatementNode.addMetaData(this.tokenizer.filePath, this.lookahead);

        letStatementNode.addChild(this.matchKeyword('let'));
        letStatementNode.addChild(this.parseVarName());
        // left off here ----
        if (this.isPropertyAccess()) {
            letStatementNode.addChild(this.matchSymbol('['));
            letStatementNode.addChild(this.parseExpression());
            letStatementNode.addChild(this.matchSymbol(']')); 
        }

        letStatementNode.addChild(this.matchSymbol('='));
        letStatementNode.addChild(this.parseExpression());
        letStatementNode.addChild(this.matchSymbol(';'));

        return letStatementNode;
    }

    parseIfStatement() {
        const ifStatementNode = new TreeNode(NodeConstants.IFSTATEMENT);
        ifStatementNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        ifStatementNode.addChild(this.matchKeyword('if'));
        ifStatementNode.addChild(this.matchSymbol('('));
        ifStatementNode.addChild(this.parseExpression());
        ifStatementNode.addChild(this.matchSymbol(')'));
        ifStatementNode.addChild(this.matchSymbol('{'));
        ifStatementNode.addChild(this.parseStatements());
        ifStatementNode.addChild(this.matchSymbol('}'));

        if (this.isElse()) {
            ifStatementNode.addChild(this.matchKeyword('else'));
            ifStatementNode.addChild(this.matchSymbol('{'));
            ifStatementNode.addChild(this.parseStatements());
            ifStatementNode.addChild(this.matchSymbol('}'));
        }

        return ifStatementNode;
    }

    parseDoStatement() {
        const doStatmentNode = new TreeNode(NodeConstants.DOSTATEMENT);
        doStatmentNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        doStatmentNode.addChild(this.matchKeyword('do'));
        doStatmentNode.addChild(this.parseSubroutineCall());
        doStatmentNode.addChild(this.matchSymbol(';'));
        return doStatmentNode;
    }

    parseReturnStatement() {
        const returnStatementNode = new TreeNode(NodeConstants.RETURNSTATEMENT);
        returnStatementNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        returnStatementNode.addChild(this.matchKeyword('return'));

        if (this.isTerm()) {
            returnStatementNode.addChild(this.parseExpression());
        }

        returnStatementNode.addChild(this.matchSymbol(';'));

        return returnStatementNode;
    }

    parseWhileStatement() {
        const whileStatementNode = new TreeNode(NodeConstants.WHILESTATEMENT);
        whileStatementNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        whileStatementNode.addChild(this.matchKeyword('while'));
        whileStatementNode.addChild(this.matchSymbol('('));
        whileStatementNode.addChild(this.parseExpression());
        whileStatementNode.addChild(this.matchSymbol(')'));
        whileStatementNode.addChild(this.matchSymbol('{'));
        whileStatementNode.addChild(this.parseStatements());
        whileStatementNode.addChild(this.matchSymbol('}'));
        return whileStatementNode;
    }

    parseExpression() {
        const expressionNode = new TreeNode(NodeConstants.EXPRESSION);
        expressionNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        expressionNode.addChild(this.parseTerm());

        while (this.isOp()) {
            expressionNode.addChild(this.parseOp());
            expressionNode.addChild(this.parseTerm());
        }

        return expressionNode;
    }

    parseTerm() {
        const termNode = new TreeNode(NodeConstants.TERM);
        termNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        if (this.isTerm()) {
            if (this.isIntegerConstant()) {
                termNode.addChild(this.parseIntegerConstant());

            } else if (this.isStringConstant()) {
                termNode.addChild(this.parseStringConstant());

            } else if (this.isKeywordConstant()) {
                termNode.addChild(this.parseKeywordConstant());

            } else if (this.isGroup()) {
                termNode.addChild(this.matchSymbol('('));
                termNode.addChild(this.parseExpression());
                termNode.addChild(this.matchSymbol(')')); 

            } else if (this.isUrnaryOp()) {
                termNode.addChild(this.parseUrnaryOp());
                termNode.addChild(this.parseTerm()); 
                
            } else if (this.isIdentifier()) {
                const [nextToken] = this.tokenizer.peekToken(); 
                // check if we perform prop access or grouping after the identifier
                if (nextToken.type === TokenConstants.SYMBOL && ['.', '(', '['].includes(nextToken.raw)) {
                    if (nextToken.raw === '(' || nextToken.raw === '.') {
                        termNode.addChild(this.parseSubroutineCall());

                    } else if (nextToken.raw === '[') {
                        termNode.addChild(this.parseVarName());
                        termNode.addChild(this.matchSymbol('['));
                        termNode.addChild(this.parseExpression());
                        termNode.addChild(this.matchSymbol(']'));
                        
                    }
                } else {
                    termNode.addChild(this.parseVarName());
                }
            }

            return termNode; 
        }

        // figure out later how to handle 'expected' value;
        this.throwParseError(this.lookahead, 'term');
    }

    parseSubroutineCall() {
        const subRoutineCallNode = new TreeNode(NodeConstants.SUBROUTINECALL);
        subRoutineCallNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        subRoutineCallNode.addChild(this.matchIdentifier());

        if (this.isGroup()) {
            subRoutineCallNode.addChild(this.matchSymbol('('));
            subRoutineCallNode.addChild(this.parseExpressionList());
            subRoutineCallNode.addChild(this.matchSymbol(')'));
            return subRoutineCallNode;
        } else if (this.isDotNotation()) {
            subRoutineCallNode.addChild(this.matchSymbol('.'));
            subRoutineCallNode.addChild(this.parseSubroutineName());
            subRoutineCallNode.addChild(this.matchSymbol('('));
            subRoutineCallNode.addChild(this.parseExpressionList());
            subRoutineCallNode.addChild(this.matchSymbol(')'));
            return subRoutineCallNode;
        }
    }

    parseUrnaryOp() {
        const urnaryOpNode = new TreeNode(NodeConstants.UNARYOP);
        urnaryOpNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        urnaryOpNode.addChild(this.matchSymbolMultiple(['-', '~']));
        return urnaryOpNode; 
    }

    parseOp() {
        const opNode = new TreeNode(NodeConstants.OP);
        opNode.addMetaData(this.tokenizer.filePath, this.lookahead);
        opNode.addChild(this.matchSymbolMultiple(['+', '-', '*', '/', '&', '|', '<', '>', '=']));
        return opNode;
    }

    parseExpressionList() {
        const expressionListNode = new TreeNode(NodeConstants.EXPRESSIONLIST);
        expressionListNode.addMetaData(this.tokenizer.filePath, this.lookahead);
    
        if (this.isGroupClosingTag()) {
            return expressionListNode;

        } else {
            expressionListNode.addChild(this.parseExpression());

            while (this.isList()) {
                expressionListNode.addChild(this.matchSymbol(','));
                expressionListNode.addChild(this.parseExpression());
            }

            return expressionListNode;
        }
    }

    parseIntegerConstant() {
        if (this.isIntegerConstant()) {
            const integerConstantNode = new TreeNode(NodeConstants.INTEGER, this.lookahead.raw);
            integerConstantNode.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken(); 
            return integerConstantNode;
        }
        this.throwParseError(this.lookahead, NodeConstants.INTEGER);
    }

    parseStringConstant() {
        if (this.isStringConstant()) {
            const stringConstantNode = new TreeNode(TokenConstants.STRING, this.lookahead.raw);
            stringConstantNode.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken();
            return stringConstantNode;
        }

        this.throwParseError(this.lookahead, NodeConstants.STRING);
    }

    parseKeywordConstant() {
        if (this.isKeywordConstant()) {
            const keyWordConstantNode = new TreeNode(NodeConstants.KEYWORDCONSTANT, this.lookahead.raw);
            keyWordConstantNode.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken(); 
            return keyWordConstantNode;
        }
        this.throwParseError(this.lookahead, ['null', 'true', 'false', 'this']);
    }

    isClassVarDec() {
        return this.lookahead.type === TokenConstants.KEYWORD && (this.lookahead.raw === 'static' || this.lookahead.raw === 'field');
    };

    isSubroutineDec() {
        return this.lookahead.type === TokenConstants.KEYWORD && (this.lookahead.raw === 'constructor' || this.lookahead.raw === 'function' || this.lookahead.raw === 'method');
    };

    isList() {
        return this.lookahead.type === TokenConstants.SYMBOL && this.lookahead.raw === ',';
    };

    isPropertyAccess() {
        return this.lookahead.type === TokenConstants.SYMBOL && this.lookahead.raw === '['; 
    }

    isStatement() {
        return this.lookahead.type === TokenConstants.KEYWORD
            && (this.lookahead.raw === 'let'
                || this.lookahead.raw === 'if'
                || this.lookahead.raw === 'while'
                || this.lookahead.raw === 'do'
                || this.lookahead.raw === 'return');
    };

    isTerm() {
        // to-do - we should keep track of the types of identifiers and validate them as we go. 
        // i.e. is it a varname or a subroutine? 
        return this.lookahead.type === TokenConstants.STRING
            || this.lookahead.type === TokenConstants.INTEGER
            || this.lookahead.type === TokenConstants.IDENTIFIER
            || this.isKeywordConstant()
            || this.isGroup()
            || this.isUrnaryOp()
    }

    isKeywordConstant() {
        return this.lookahead.type === TokenConstants.KEYWORD
            && (this.lookahead.raw === 'true'
            || this.lookahead.raw === 'false'
            || this.lookahead.raw === 'null'
            || this.lookahead.raw === 'this')
    }

    isVarDec() {
        return this.lookahead.type === TokenConstants.KEYWORD && this.lookahead.raw === 'var';
    }

    isVoid() {
        return this.lookahead.type === TokenConstants.KEYWORD && this.lookahead.raw === 'void'; 
    }

    isGroup() {
        return this.lookahead.type === TokenConstants.SYMBOL && this.lookahead.raw === '(';
    }

    isGroupClosingTag() {
        return this.lookahead.type === TokenConstants.SYMBOL && this.lookahead.raw === ')'; 
    }

    isUrnaryOp() {
        return this.lookahead.type === TokenConstants.SYMBOL && (this.lookahead.raw === '-' || this.lookahead.raw === '~');
    }

    isOp() {
        return this.lookahead.type === TokenConstants.SYMBOL
            && ['+', '-', '*', '/', '&', '|', '<', '>', '='].includes(this.lookahead.raw); 
    }

    isDotNotation() {
        return this.lookahead.type === TokenConstants.SYMBOL && this.lookahead.raw === '.';
    }

    isStringConstant() {
        return this.lookahead.type === TokenConstants.STRING;
    }

    isIntegerConstant() {
        return this.lookahead.type === TokenConstants.INTEGER;
    }

    isIdentifier() {
        return this.lookahead.type === TokenConstants.IDENTIFIER;
    }

    isElse() {
        return this.lookahead.type === TokenConstants.KEYWORD && this.lookahead.raw === 'else';
    }

    matchKeyword(keyword) {
        if (this.lookahead.type === NodeConstants.KEYWORD && this.lookahead.raw === keyword) {
            const node = new TreeNode(NodeConstants.KEYWORD, this.lookahead.raw);
            node.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken();
            return node;
        }
        this.throwParseError(this.lookahead, keyword)
    }

    matchKeywordMultiple(keywords) {
        if (this.lookahead.type === NodeConstants.KEYWORD && keywords.some(key => this.lookahead.raw === key)) {
            const node = new TreeNode(NodeConstants.KEYWORD, this.lookahead.raw);
            node.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken();
            return node;
        }
        this.throwParseError(this.lookahead, JSON.stringify(keywords));
    }

    matchSymbol(symbol) {
        if (this.lookahead.type === TokenConstants.SYMBOL && this.lookahead.raw === symbol) {
            const node = new TreeNode(TokenConstants.SYMBOL, this.lookahead.raw);
            node.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken();
            return node;
        }
        this.throwParseError(this.lookahead, symbol);
    }

    matchSymbolMultiple(symbols) {
        if (this.lookahead.type === TokenConstants.SYMBOL && symbols.some(sym => this.lookahead.raw === sym)) {
            const node = new TreeNode(TokenConstants.SYMBOL, this.lookahead.raw);
            node.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken();
            return node;
        }
        this.throwParseError(this.lookahead, JSON.stringify(symbols)); 
    }

    matchIdentifier() {
        if (this.lookahead.type === NodeConstants.IDENTIFIER) {
            const node = new TreeNode(NodeConstants.IDENTIFIER, this.lookahead.raw);
            node.addMetaData(this.tokenizer.filePath, this.lookahead);
            this.consumeToken();
            return node;
        }
        this.throwParseError(this.lookahead, TokenConstants.IDENTIFIER);
    }

    consumeToken() {
        this.lookahead = this.tokenizer.getNextToken();
    }

    /**
     * 
     * @param {Token} token 
     * @param {*} expected 
     */
    throwParseError(token, expected) {
        throw new Error(`Unexpected token "${token.raw}" at ${this.tokenizer.filePath} ${token.lineCount}:${token.charCount}`)
    }
}