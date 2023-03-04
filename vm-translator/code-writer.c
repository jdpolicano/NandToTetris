#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
#include <math.h>
#include <stdarg.h>
#include "parser.h"

static int COMP_EQ_COUNT = 1;
static int COMP_GT_COUNT = 1;
static int COMP_LT_COUNT = 1;
static int RETURN_COUNTER = 1;
static int FRAME_SIZE = 5;
static int HAVE_WRITTEN_BOOTSTRAP = 0;
static char *FUNCTION_CONTEXT;
static char *GLOBAL_CONTEXT;
static char *FILE_NAME;
static FILE* OUTPUT_FILE;
static char *TEMP_REGISTER = "R13";
static char *FRAME_REGISTER = "R14";
static char *RETURN_ADDRESS_REGISTER = "R15";
// might want to ocnsider returning an integer to indicate success or failure, but for now strategy is to exit program an print error if we hit a bump.
void translate(TOKEN_ARRAY* tokens, FILE* );

// Bootstrapping
void write_bootstrap(void);

// Identifiers 
int is_pop(char* type);
int is_push(char* type);
int is_virtual(char* segement);
int is_math(char* type);
int is_label(char* type);
int is_goto(char* type);
int is_if_goto(char* type);
int is_function(char* type);
int is_return(char* type);
int is_call(char* type);

// branching
void write_label(char *label);
void write_goto(char *label);
void write_if_goto(char* label);

// function calls
void write_function(char* function_name, char* num_locals);
void write_return(void);
void write_call(char* function_name, char* num_args);

// pop writers
void route_pop(TOKEN pop_token);
void pop_virtual(char* segement, char* index);
void pop_static(char* index);
void pop_temp(char* index);
void pop_pointer(char* index);

// Push writers
void route_push(TOKEN push_token);
void push_constant(char* value);
void push_virtual(char* segement, char* index);
void push_static(char* index);
void push_temp(char* index);
void push_pointer(char* index);
void push_register(char* register_name);

// Maths
void route_math(TOKEN math_token);
void add(void);
void subtract(void);
void negate(void);
void equality(void);
void greater_than(void);
void less_than(void);
void bit_and(void);
void bit_or(void);
void bit_not(void);

// Primitives for writing
void format_label(char* label, char* destination);
void push_top(void);
void pop_top(void);
void increment_sp(void);
void decrement_sp(void);
void write_address(char *destination);
void write_cpu(char* destination, char* source);
void write_comment(TOKEN token);
void write_jump(char *destination, char *jump);
void write_template(char *template, ...);
void write(char* string);


void translate(TOKEN_ARRAY* tokens, FILE* output_file) {

    // Setup global variables
    FILE_NAME = tokens->file_name;
    // starting context should be filename.global
    GLOBAL_CONTEXT = malloc(strlen(FILE_NAME) + 10);
    if (GLOBAL_CONTEXT == NULL) {
        printf("Error in %s.vm :: could not allocate memory for CONTEXT\n", FILE_NAME);
        exit(1);
    }
    strcpy(GLOBAL_CONTEXT, FILE_NAME);
    strcat(GLOBAL_CONTEXT, ".__GLOBAL__");
    // Starting function context as global. this will be swtiched back and forth as we enter and exit functions
    FUNCTION_CONTEXT = GLOBAL_CONTEXT;
    // Make output file globally accessible.
    OUTPUT_FILE = output_file;

    // Write bootstrap code
    if (!HAVE_WRITTEN_BOOTSTRAP) {
        write_bootstrap();
        HAVE_WRITTEN_BOOTSTRAP = 1;
    }

    // Iterate through tokens and write code
    for (int i = 0; i < tokens->length; i++) {
        TOKEN curr = tokens->tokens[i];
        write_comment(curr);

        if (is_math(curr.type)) {
            route_math(curr);
        } else if (is_push(curr.type)) {
            route_push(curr);
        } else if (is_pop(curr.type)) {
            route_pop(curr);
        } else if (is_label(curr.type)) {
            write_label(curr.arg1);
        } else if (is_goto(curr.type)) {
            write_goto(curr.arg1);
        } else if (is_if_goto(curr.type)) {
            write_if_goto(curr.arg1);
        } else if (is_function(curr.type)) {
            write_function(curr.arg1, curr.arg2); 
        } else if (is_return(curr.type)) {
            write_return();
        } else if (is_call(curr.type)) {
            write_call(curr.arg1, curr.arg2);
        } else {
            printf("Error in %s.vm at line %i :: unexpected type %s\n", FILE_NAME, curr.lineNum, curr.type);
            exit(1);
        }
    }

    free(GLOBAL_CONTEXT);
}


//////////////////////// BOOTSTRAPPING ////////////////////////
void write_bootstrap(void) {
    // SP = 256
    write_address("256");
    write_cpu("D", "A");
    write_address("SP");
    write_cpu("M", "D");

    // Call Sys.init
    write_call("Sys.init", "0");
    return;
};

//////////////////////// Branching ////////////////////////
void write_label(char* label) {
    char formatted_label[100];
    format_label(label, formatted_label);
    write_template("(%s)\n", formatted_label);
}

void write_goto(char* label) {
    char formatted_label[100];
    format_label(label, formatted_label);
    write_address(formatted_label);
    write_jump("0", "JMP");
}

void write_if_goto(char* label) {
    char formatted_label[100];
    format_label(label, formatted_label);
    pop_top();
    write_address(formatted_label);
    write_jump("D", "JNE");
}
//////////////////////// Branching ////////////////////////

//////////////////////// FUNCTIONS ////////////////////////

void write_function(char* function_name, char* num_locals) {
    FUNCTION_CONTEXT = function_name; // this will be switched back after next return statement.
    write_template("(%s)\n", function_name);
    for (int i = 0; i < atoi(num_locals); i++) {
        write_template("// local var # %i\n", i + 1);
        push_constant("0");
    }
    return;
}

void write_call(char* function_name, char* num_args) {
    // push return address (i.e., foo.main%ret.1);
    write_template("@%s$ret.%i\n", FUNCTION_CONTEXT, RETURN_COUNTER);
    write_cpu("D", "A"); // D = return address
    push_top();
    // push LCL
    push_register("LCL");
    // push ARG
    push_register("ARG");
    // push THIS
    push_register("THIS");
    // push THAT
    push_register("THAT");
    // LCL = SP
    write_address("SP");
    write_cpu("D", "M");
    write_address("LCL");
    write_cpu("M", "D");
    // ARG = SP - 5 - nArgs
    write_template("@%i\n", 5 + atoi(num_args)); // nArgs + 5
    write_cpu("D", "D-A"); // D = SP - 5 - nArgs
    write_address("ARG");
    write_cpu("M", "D"); // ARG = SP - 5 - nArgs
    // goto f
    write_address(function_name);
    write_jump("0", "JMP");
    // (return-address)
    write_template("(%s$ret.%i)\n", FUNCTION_CONTEXT, RETURN_COUNTER);
    RETURN_COUNTER++;
}

void write_return(void) {
    // FRAME = LCL
    write_address("LCL");
    write_cpu("D", "M");
    write_address(FRAME_REGISTER);
    write_cpu("M", "D");
    // RETURN_REGISTER = *(FRAME - 5)
    write_template("@%i\n", 5);
    write_cpu("D", "A");
    write_address(FRAME_REGISTER);
    write_cpu("A", "M-D");
    write_cpu("D", "M");
    write_address(RETURN_ADDRESS_REGISTER);
    write_cpu("M", "D");
    // *ARG[0] = pop()
    pop_top();
    write_address("ARG");
    write_cpu("A", "M");
    write_cpu("M", "D");
    // SP = ARG + 1
    write_address("ARG");
    write_cpu("D", "M");
    write_address("SP");
    write_cpu("M", "D+1");

    char* virtual_segements[] = {"THAT", "THIS", "ARG", "LCL"};

    for (int i = 0; i < 4; i++) {
        // *(FRAME - i) = *(FRAME - (i + 1))
        write_template("@%i\n", i + 1);
        write_cpu("D", "A");
        write_address(FRAME_REGISTER);
        write_cpu("A", "M-D");
        write_cpu("D", "M");
        write_address(virtual_segements[i]);
        write_cpu("M", "D");
    }

    // goto RETURN_ADDRESS
    write_address(RETURN_ADDRESS_REGISTER);
    write_cpu("A", "M");
    write_jump("0", "JMP");
    FUNCTION_CONTEXT = GLOBAL_CONTEXT; // switch back to global context
}
//////////////////////// FUNCTIONS ////////////////////////

//////////////////////// POP COMMANDS ////////////////////////
void route_pop(TOKEN pop_token) {
    char *segment = pop_token.arg1;
    char *index = pop_token.arg2;
    int lineNum = pop_token.lineNum;

    if (segment == NULL || index == NULL) {
        printf("Error in token at line %i :: expected char* but received NULL\n", lineNum);
        exit(1);
    } if (is_virtual(segment)) {
        pop_virtual(segment, index);
    } else if (strcmp(segment, "pointer") == 0) {
        pop_pointer(index);
    } else if (strcmp(segment, "temp") == 0) {
        pop_temp(index);
    } else if (strcmp(segment, "static") == 0) {
        pop_static(index);
    } else {
        printf("Error in token at line %i :: unexpected segement %s\n", lineNum, segment);
        exit(1);
    }
}

void pop_virtual(char* segement, char* index) {
       char* target;

        if (strcmp(segement, "this") == 0) target = "THIS";
        if (strcmp(segement, "that") == 0) target = "THAT";
        if (strcmp(segement, "local") == 0) target = "LCL";
        if (strcmp(segement, "argument") == 0) target = "ARG";
        
        write_address(index);  // @[index]
        write_cpu("D", "A"); // D=A
        write_address(target); // @[target]
        write_cpu("M", "D+M"); // M=D+M
        pop_top(); // see pop routine
        write_address(target); // @[target]
        write_cpu("A", "M"); // A=M
        write_cpu("M", "D"); // M=D
        write_address(index);  // @[index]
        write_cpu("D", "A"); // D=A
        write_address(target); // @[target]
        write_cpu("M", "M-D"); // M=M-D
}

void pop_pointer(char* index) {
    char* target;

    if (strcmp(index, "0") == 0) target = "THIS";
    if (strcmp(index, "1") == 0) target = "THAT";

    pop_top();
    write_address(target); // @[target]
    write_cpu("M", "D"); // M=D
}

void pop_temp(char* index) {
    int temp_address = 5 + atoi(index); // 8 temps starting from 5 index, so 12 is the max. 
    char* temp_address_str = malloc(sizeof(char) * 10);
    sprintf(temp_address_str, "%i", temp_address);

    pop_top(); // @SP // AM=M-1 // D=M 
    write_address(temp_address_str); // @[temp + index]
    write_cpu("M", "D"); // M=D
    free(temp_address_str);
}

void pop_static(char* index) {
    char* static_address = malloc(sizeof(char) * 100);
    sprintf(static_address, "%s.%s", FILE_NAME, index);
    pop_top(); // @SP // AM=M-1 // D=M 
    write_address(static_address); // @[static_address]
    write_cpu("M", "D"); // M=D
    free(static_address);
}
//////////////////////// POP COMMANDS ////////////////////////
//////////////////////// PUSH COMMANDS ////////////////////////
void route_push(TOKEN push_token) {
    char* segment = push_token.arg1;
    char* index = push_token.arg2;
    int lineNum = push_token.lineNum;

    if (segment == NULL || index == NULL) {
        printf("Error in token at line %i :: expected char* but received NULL\n", lineNum);
        exit(1); 
    }
    
    if (is_virtual(segment)) {
        push_virtual(segment, index);
    } else if (strcmp(segment, "constant") == 0) {
        push_constant(index);
    } else if (strcmp(segment, "pointer") == 0) {
        push_pointer(index);
    } else if (strcmp(segment, "temp") == 0) {
        push_temp(index);
    } else if (strcmp(segment, "static") == 0) {
        push_static(index);
    } else {
        printf("Error in token at line %i :: unexpected segement %s\n", lineNum, segment);
        exit(1); 
    }
}

// Pushes an item from a virtual array onto the stack. Requires an index for the loacation to pish to.
void push_virtual(char* segement, char* index) {
    char* target;

    if (strcmp(segement, "this") == 0) target = "THIS";
    if (strcmp(segement, "that") == 0) target = "THAT";
    if (strcmp(segement, "local") == 0) target = "LCL";
    if (strcmp(segement, "argument") == 0) target = "ARG";

    write_address(index);  // @[index]
    write_cpu("D", "A"); // D=A
    write_address(target); // @[target]
    write_cpu("A", "D+M"); // A=D+M
    write_cpu("D", "M"); // D=M
    push_top(); // see push routine
}

// Pushes an item from a virtual array. Requires an index for the loacation to pish to.
void push_pointer(char* index) {
    char* target;

    if (strcmp(index, "0") == 0) target = "THIS";
    if (strcmp(index, "1") == 0) target = "THAT";

    write_address(target); // @[target]
    write_cpu("D", "M"); // D=M
    push_top(); // see push routine
}

void push_temp(char* index) {
    int temp_address = 5 + atoi(index); // 8 temps starting from 5 index, so 12 is the max. 
    char* temp_address_str = malloc(sizeof(char) * 10);
    sprintf(temp_address_str, "%i", temp_address);

    write_address(temp_address_str); // @[temp_address]
    write_cpu("D", "M"); // D=M
    push_top(); // see push routine
    free(temp_address_str);
}

void push_constant(char* value) {
    write_address(value); // @[value]
    write_cpu("D", "A"); // D=A;
    push_top();
}

void push_static(char* index) {
    char* static_address = malloc(sizeof(char) * 10);
    sprintf(static_address, "%s.%s", FILE_NAME, index);
    write_address(static_address); // @[static_address]
    write_cpu("D", "M"); // D=M
    push_top(); // see push routine
    free(static_address);
}

void push_register(char* register_name) {
    write_template("// push_register %s\n", register_name);
    write_address(register_name); // @[register_name]
    write_cpu("D", "M"); // D=M
    push_top(); // see push routine
}
//////////////////////// PUSH COMMANDS ////////////////////////


//////////////////////// MATH OPERTATIONS /////////////////////
void route_math(TOKEN math_token) {
    char* operation = math_token.arg1;

    if (strcmp(operation, "add") == 0) {
        add(); 
    } else if (strcmp(operation, "sub") == 0) {
        subtract();
    } else if (strcmp(operation, "neg") == 0) {
        negate();
    } else if (strcmp(operation, "eq") == 0) {
        equality();
    } else if (strcmp(operation, "gt") == 0) {
        greater_than();
    } else if (strcmp(operation, "lt") == 0) {
        less_than();
    } else if (strcmp(operation, "and") == 0) {
        bit_and();
    } else if (strcmp(operation, "or") == 0) {
        bit_or();
    } else if (strcmp(operation, "not") == 0) {
        bit_not();
    }
}

void add(void) {
    pop_top();
    decrement_sp();
    write_cpu("M", "M+D"); // M=M+D
    increment_sp();
}


void subtract(void) {
    pop_top();
    decrement_sp();
    write_cpu("M", "M-D"); // D=M-D
    increment_sp();
}

void negate(void) {
    decrement_sp(); 
    write_cpu("M", "-M"); // M=-M
    increment_sp();
}


void equality(void) {
    pop_top();
    decrement_sp(); 
    write_cpu("D", "M-D"); // D=M-D
    write_template("@ARITHMATIC.eq.%d.IF_TRUE\n", COMP_EQ_COUNT);
    write_jump("D", "JEQ"); // if (D == 0) goto ARITHMATIC.eq.%d.IF_TRUE
    write_cpu("D", "0"); // D=0
    write_template("@ARITHMATIC.eq.%d.END\n", COMP_EQ_COUNT); // @ARITHMATIC.eq.%d.END
    write_jump("0", "JMP"); // goto ARITHMATIC.eq.%d.END
    write_template("(ARITHMATIC.eq.%d.IF_TRUE)\n", COMP_EQ_COUNT); // (ARITHMATIC.eq.%d.IF_TRUE)
    write_cpu("D", "-1"); // D=-1
    write_template("(ARITHMATIC.eq.%d.END)\n", COMP_EQ_COUNT); // (ARITHMATIC.eq.%d.END)
    push_top();
    COMP_EQ_COUNT++;
}

void greater_than(void) {
    pop_top();
    decrement_sp();
    write_cpu("D", "M-D"); // D=M-D
    write_template("@ARITHMATIC.gt.%d.IF_TRUE\n", COMP_GT_COUNT); // @ARITHMATIC.gt.%d.IF_TRUE
    write_jump("D", "JGT"); // if (D > 0) goto ARITHMATIC.gt.%d.IF_TRUE
    write_cpu("D", "0"); // D=0
    write_template("@ARITHMATIC.gt.%d.END\n", COMP_GT_COUNT); // @ARITHMATIC.gt.%d.END
    write_jump("0", "JMP"); // goto ARITHMATIC.gt.%d.END
    write_template("(ARITHMATIC.gt.%d.IF_TRUE)\n", COMP_GT_COUNT); // (ARITHMATIC.gt.%d.IF_TRUE)
    write_cpu("D", "-1"); // D=-1
    write_template("(ARITHMATIC.gt.%d.END)\n", COMP_GT_COUNT); // (ARITHMATIC.gt.%d.END)
    push_top(); 
    COMP_GT_COUNT++;
}

void less_than(void) {
    pop_top();
    decrement_sp();
    write_cpu("D", "M-D"); // D=M-D
    write_template("@ARITHMATIC.lt.%d.IF_TRUE\n", COMP_LT_COUNT); // @ARITHMATIC.lt.%d.IF_TRUE
    write_jump("D", "JLT"); // if (D < 0) goto ARITHMATIC.lt.%d.IF_TRUE
    write_cpu("D", "0"); // D=0
    write_template("@ARITHMATIC.lt.%d.END\n", COMP_LT_COUNT); // @ARITHMATIC.lt.%d.END
    write_jump("0", "JMP"); // goto ARITHMATIC.lt.%d.END
    write_template("(ARITHMATIC.lt.%d.IF_TRUE)\n", COMP_LT_COUNT); // (ARITHMATIC.lt.%d.IF_TRUE)
    write_cpu("D", "-1"); // D=-1
    write_template("(ARITHMATIC.lt.%d.END)\n", COMP_LT_COUNT); // (ARITHMATIC.lt.%d.END)
    push_top();
    COMP_LT_COUNT++;
}

void bit_and(void) {
    pop_top();
    decrement_sp();
    write_cpu("M", "M&D"); // D=M&D
    increment_sp();
}

void bit_or(void) {
    pop_top();
    decrement_sp();
    write_cpu("M", "M|D"); // D=M|D
    increment_sp();
}

void bit_not(void) {
    decrement_sp();
    write_cpu("M", "!M"); // D=!M
    increment_sp();
}
//////////////////////// MATH OPERTATIONS /////////////////////
//////////////////////// PRIM OPERTATIONS /////////////////////
// takes a label and formats it using the global FILE_NAME and CONTEXT variable
void format_label(char* label, char* destination) {
    sprintf(destination, "%s$%s", FUNCTION_CONTEXT, label);
}


void pop_top(void) {
    decrement_sp();
    write_cpu("D", "M"); // D=M
    return;
}

// Pushes data reg to top of stack;
void push_top(void) {
    write_address("SP"); // @SP
    write_cpu("A", "M"); // A=M
    write_cpu("M", "D"); // M=D
    increment_sp();
    return;
}

// increments the stack pointer and points address reg at virtual address
void increment_sp(void) {
    write_address("SP"); // @SP
    write_cpu("AM", "M+1"); // A=M+1
    return;
}

// decrements the stack pointer and points address reg at virtual address.
void decrement_sp(void) {
    write_address("SP"); // @SP
    write_cpu("AM", "M-1"); // A=M-1
    return;
}

void write_comment(TOKEN token) {
    write_template("// %s %s %s\n", token.type, token.arg1, token.arg2);
    return;
}

void write_cpu(char* destination, char* source) {
    write_template("%s=%s\n", destination, source);
}

void write_address(char* destination) {
    write_template("@%s\n", destination);
}

void write_jump(char* destination, char* jump) {
    write_template("%s;%s\n", destination, jump);
}

void write_template(char *template, ...) {
    va_list args;
    va_start(args, template);
    char buffer[128]; 
    vsprintf(buffer, template, args);
    write(buffer);
    va_end(args);
}

void write(char* string) {
    fwrite(string, sizeof(char), strlen(string), OUTPUT_FILE);
    return;
}
//////////////////////// PRIM OPERTATIONS /////////////////////

//////////////////////////// IDENTIFIERS ///////////////////////////
int is_math(char* type) {
    return strcmp(type, C_MATH) == 0;
}

int is_pop(char* type) {
    return strcmp(type, C_POP) == 0;
}

int is_push(char* type) {
    return strcmp(type, C_PUSH) == 0;
}

int is_virtual(char* segement) {
    char* virtual_addresses[] = { "this", "that", "argument", "local" };
    int num_addresses = 4;

    for (int i = 0; i < num_addresses; i++) {
        if (strcmp(virtual_addresses[i], segement) == 0) {
            return 1;
        }
    }
    
    return 0; 
}

int is_label(char* type) {
    return strcmp(type, C_LABEL) == 0;
}

int is_goto(char* type) {
    return strcmp(type, C_GOTO) == 0;
}

int is_if_goto(char* type) {
    return strcmp(type, C_IF_GOTO) == 0;
}

int is_function(char* type) {
    return strcmp(type, C_FUNC) == 0;
}

int is_return(char* type) {
    return strcmp(type, C_RETURN) == 0;
}

int is_call(char* type) {
    return strcmp(type, C_CALL) == 0;
}
//////////////////////// IDENTIFIERS //////////////////////////