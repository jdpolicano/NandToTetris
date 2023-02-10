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
// might want to ocnsider returning an integer to indicate success or failure, but for now strategy is to exit program an print error if we hit a bump.
void translate(TOKEN_ARRAY* tokens, FILE* output_file);

// Identifiers 
int is_pop(char* type);
int is_push(char* type);
int is_virtual(char* segement);
int is_math(char* type);

// pop writers
void route_pop(TOKEN pop_token, char* file_name, FILE* output_file);
void pop_virtual(char* segement, char* index, FILE* output_file);
void pop_static(char* index, char* file_name, FILE* output_file);
void pop_temp(char* index, FILE* output_file);
void pop_pointer(char* index, FILE* output_file);
// Push writers
void route_push(TOKEN push_token, char* file_name, FILE* output_file);
void push_constant(char* value, FILE* output_file);
void push_virtual(char* segement, char* index, FILE* output_file);
void push_static(char* index, char* file_name, FILE* output_file);
void push_temp(char* index, FILE* output_file);
void push_pointer(char* index, FILE* output_file);
// Maths
void route_math(TOKEN math_token, FILE* output_file);
void add(FILE* output_file);
void subtract(FILE* output_file);
void negate(FILE* output_file);
void equality(FILE* output_file);
void greater_than(FILE* output_file);
void less_than(FILE* output_file);
void bit_and(FILE* output_file);
void bit_or(FILE* output_file);
void bit_not(FILE* output_file);
// Primitives for writing
void push_top(FILE* output_file);
void pop_top(FILE* output_file);
void increment_sp(FILE* output_file);
void decrement_sp(FILE* output_file);
void write_address(FILE *output_file, char *destination);
void write_cpu(FILE *output_file, char* destination, char* source);
void write_comment(FILE *output_file, TOKEN token);
void write_jump(FILE *output_file, char *destination, char *jump);
void write_template(FILE *output_file, char *template, ...);
void write(FILE* output_file, char* string);


void translate(TOKEN_ARRAY* tokens, FILE* output_file) {
    char* file_name = tokens->file_name;

    for (int i = 0; i < tokens->length; i++) {
        TOKEN curr = tokens->tokens[i];
        if (is_math(curr.type)) {
            route_math(curr, output_file);
        } else if (is_push(curr.type)) {
            route_push(curr, file_name, output_file);
        } else if (is_pop(curr.type)) {
            route_pop(curr, file_name, output_file);
        }
    }
}

//////////////////////////// IDENTIFIERS ///////////////////////////
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


//////////////////////// IDENTIFIERS //////////////////////////
//////////////////////// POP COMMANDS ////////////////////////
void route_pop(TOKEN pop_token, char* file_name, FILE* output_file) {
    char* segment = pop_token.arg1;
    char* index = pop_token.arg2;
    int lineNum = pop_token.lineNum;

    if (segment == NULL || index == NULL) {
        printf("Error in token at line %i :: expected char* but received NULL\n", lineNum);
        exit(1); 
    }

    write_comment(output_file, pop_token);

    if (is_virtual(segment)) {
        pop_virtual(segment, index, output_file);
    } else if (strcmp(segment, "pointer") == 0) {
        pop_pointer(index, output_file);
    } else if (strcmp(segment, "temp") == 0) {
        pop_temp(index, output_file);
    } else if (strcmp(segment, "static") == 0) {
        pop_static(index, file_name, output_file);
    } else {
        printf("Error in token at line %i :: unexpected segement %s\n", lineNum, segment);
        exit(1); 
    }
}

void pop_virtual(char* segement, char* index, FILE* output_file) {
       char* target;

        if (strcmp(segement, "this") == 0) target = "THIS";
        if (strcmp(segement, "that") == 0) target = "THAT";
        if (strcmp(segement, "local") == 0) target = "LCL";
        if (strcmp(segement, "argument") == 0) target = "ARG";
        
        write_address(output_file, index);  // @[index]
        write_cpu(output_file, "D", "A"); // D=A
        write_address(output_file, target); // @[target]
        write_cpu(output_file, "M", "D+M"); // M=D+M
        pop_top(output_file); // see pop routine
        write_address(output_file, target); // @[target]
        write_cpu(output_file, "A", "M"); // A=M
        write_cpu(output_file, "M", "D"); // M=D
        write_address(output_file, index);  // @[index]
        write_cpu(output_file, "D", "A"); // D=A
        write_address(output_file, target); // @[target]
        write_cpu(output_file, "M", "M-D"); // M=M-D
}

void pop_pointer(char* index, FILE* output_file) {
    char* target;

    if (strcmp(index, "0") == 0) target = "THIS";
    if (strcmp(index, "1") == 0) target = "THAT";

    pop_top(output_file);
    write_address(output_file, target); // @[target]
    write_cpu(output_file, "M", "D"); // M=D
}

void pop_temp(char* index, FILE* output_file) {
    int temp_address = 5 + atoi(index); // 8 temps starting from 5 index, so 12 is the max. 
    char* temp_address_str = malloc(sizeof(char) * 10);
    sprintf(temp_address_str, "%i", temp_address);

    pop_top(output_file); // @SP // AM=M-1 // D=M 
    write_address(output_file, temp_address_str); // @[temp + index]
    write_cpu(output_file, "M", "D"); // M=D
    free(temp_address_str);
}

void pop_static(char* index, char* file_name, FILE* output_file) {
    char* static_address = malloc(sizeof(char) * 100);
    sprintf(static_address, "%s.%s", file_name, index);

    pop_top(output_file); // @SP // AM=M-1 // D=M 
    write_address(output_file, static_address); // @[static_address]
    write_cpu(output_file, "M", "D"); // M=D
    free(static_address);
}
//////////////////////// POP COMMANDS ////////////////////////
//////////////////////// PUSH COMMANDS ////////////////////////
void route_push(TOKEN push_token, char* file_name, FILE* output_file) {
    char* segment = push_token.arg1;
    char* index = push_token.arg2;
    int lineNum = push_token.lineNum;

    if (segment == NULL || index == NULL) {
        printf("Error in token at line %i :: expected char* but received NULL\n", lineNum);
        exit(1); 
    }

    write_comment(output_file, push_token);
    
    if (is_virtual(segment)) {
        push_virtual(segment, index, output_file);
    } else if (strcmp(segment, "constant") == 0) {
        push_constant(index, output_file);
    } else if (strcmp(segment, "pointer") == 0) {
        push_pointer(index, output_file);
    } else if (strcmp(segment, "temp") == 0) {
        push_temp(index, output_file);
    } else if (strcmp(segment, "static") == 0) {
        push_static(index, file_name, output_file);
    } else {
        printf("Error in token at line %i :: unexpected segement %s\n", lineNum, segment);
        exit(1); 
    }
}

void push_virtual(char* segement, char* index, FILE* output_file) {
    char* target;

    if (strcmp(segement, "this") == 0) target = "THIS";
    if (strcmp(segement, "that") == 0) target = "THAT";
    if (strcmp(segement, "local") == 0) target = "LCL";
    if (strcmp(segement, "argument") == 0) target = "ARG";

    write_address(output_file, index);  // @[index]
    write_cpu(output_file, "D", "A"); // D=A
    write_address(output_file, target); // @[target]
    write_cpu(output_file, "A", "D+M"); // A=D+M
    write_cpu(output_file, "D", "M"); // D=M
    push_top(output_file); // see push routine
}

void push_pointer(char* index, FILE* output_file) {
    char* target;

    if (strcmp(index, "0") == 0) target = "THIS";
    if (strcmp(index, "1") == 0) target = "THAT";

    write_address(output_file, target); // @[target]
    write_cpu(output_file, "D", "M"); // D=M
    push_top(output_file); // see push routine
}

void push_temp(char* index, FILE* output_file) {
    int temp_address = 5 + atoi(index); // 8 temps starting from 5 index, so 12 is the max. 
    char* temp_address_str = malloc(sizeof(char) * 10);
    sprintf(temp_address_str, "%i", temp_address);

    write_address(output_file, temp_address_str); // @[temp_address]
    write_cpu(output_file, "D", "M"); // D=M
    push_top(output_file); // see push routine
    free(temp_address_str);
}

void push_constant(char* value, FILE* output_file) {
    write_address(output_file, value); // @[value]
    write_cpu(output_file, "D", "A"); // D=A;
    push_top(output_file);
}

void push_static(char* index, char* file_name, FILE* output_file) {
    char* static_address = malloc(sizeof(char) * 10);
    sprintf(static_address, "%s.%s", file_name, index);
    write_address(output_file, static_address); // @[static_address]
    write_cpu(output_file, "D", "M"); // D=M
    push_top(output_file); // see push routine
    free(static_address);
}
//////////////////////// PUSH COMMANDS ////////////////////////
//////////////////////// MATH OPERTATIONS /////////////////////
int is_math(char* type) {
    return strcmp(type, C_MATH) == 0;
}

void route_math(TOKEN math_token, FILE* output_file) {
    char* operation = math_token.arg1;

    if (strcmp(operation, "add") == 0) {
        write_comment(output_file, math_token);
        add(output_file); 
    } else if (strcmp(operation, "sub") == 0) {
        write_comment(output_file, math_token);
        subtract(output_file);
    } else if (strcmp(operation, "neg") == 0) {
        write_comment(output_file, math_token);
        negate(output_file);
    } else if (strcmp(operation, "eq") == 0) {
        write_comment(output_file, math_token);
        equality(output_file);
    } else if (strcmp(operation, "gt") == 0) {
        write_comment(output_file, math_token);
        greater_than(output_file);
    } else if (strcmp(operation, "lt") == 0) {
        write_comment(output_file, math_token);
        less_than(output_file);
    } else if (strcmp(operation, "and") == 0) {
        write_comment(output_file, math_token);
        bit_and(output_file);
    } else if (strcmp(operation, "or") == 0) {
        write_comment(output_file, math_token);
        bit_or(output_file);
    } else if (strcmp(operation, "not") == 0) {
        write_comment(output_file, math_token);
        bit_not(output_file);
    }
}

void add(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write_cpu(output_file, "M", "M+D"); // M=M+D
    increment_sp(output_file);
}


void subtract(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write_cpu(output_file, "M", "M-D"); // D=M-D
    increment_sp(output_file);
}

void negate(FILE* output_file) {
    decrement_sp(output_file); 
    write_cpu(output_file, "M", "-M"); // M=-M
    increment_sp(output_file);
}


void equality(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file); 
    write_cpu(output_file, "D", "M-D"); // D=M-D
    write_template(output_file, "@ARITHMATIC.eq.%d.IF_TRUE\n", COMP_EQ_COUNT);
    write_jump(output_file, "D", "JEQ"); // if (D == 0) goto ARITHMATIC.eq.%d.IF_TRUE
    write_cpu(output_file, "D", "0"); // D=0
    write_template(output_file, "@ARITHMATIC.eq.%d.END\n", COMP_EQ_COUNT); // @ARITHMATIC.eq.%d.END
    write_jump(output_file, "0", "JMP"); // goto ARITHMATIC.eq.%d.END
    write_template(output_file, "(ARITHMATIC.eq.%d.IF_TRUE)\n", COMP_EQ_COUNT); // (ARITHMATIC.eq.%d.IF_TRUE)
    write_cpu(output_file, "D", "-1"); // D=-1
    write_template(output_file, "(ARITHMATIC.eq.%d.END)\n", COMP_EQ_COUNT); // (ARITHMATIC.eq.%d.END)
    push_top(output_file);
    COMP_EQ_COUNT++;
}

void greater_than(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write_cpu(output_file, "D", "M-D"); // D=M-D
    write_template(output_file, "@ARITHMATIC.gt.%d.IF_TRUE\n", COMP_GT_COUNT); // @ARITHMATIC.gt.%d.IF_TRUE
    write_jump(output_file, "D", "JGT"); // if (D > 0) goto ARITHMATIC.gt.%d.IF_TRUE
    write_cpu(output_file, "D", "0"); // D=0
    write_template(output_file, "@ARITHMATIC.gt.%d.END\n", COMP_GT_COUNT); // @ARITHMATIC.gt.%d.END
    write_jump(output_file, "0", "JMP"); // goto ARITHMATIC.gt.%d.END
    write_template(output_file, "(ARITHMATIC.gt.%d.IF_TRUE)\n", COMP_GT_COUNT); // (ARITHMATIC.gt.%d.IF_TRUE)
    write_cpu(output_file, "D", "-1"); // D=-1
    write_template(output_file, "(ARITHMATIC.gt.%d.END)\n", COMP_GT_COUNT); // (ARITHMATIC.gt.%d.END)
    push_top(output_file); 
    COMP_GT_COUNT++;
}

void less_than(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write_cpu(output_file, "D", "M-D"); // D=M-D
    write_template(output_file, "@ARITHMATIC.lt.%d.IF_TRUE\n", COMP_LT_COUNT); // @ARITHMATIC.lt.%d.IF_TRUE
    write_jump(output_file, "D", "JLT"); // if (D < 0) goto ARITHMATIC.lt.%d.IF_TRUE
    write_cpu(output_file, "D", "0"); // D=0
    write_template(output_file, "@ARITHMATIC.lt.%d.END\n", COMP_LT_COUNT); // @ARITHMATIC.lt.%d.END
    write_jump(output_file, "0", "JMP"); // goto ARITHMATIC.lt.%d.END
    write_template(output_file, "(ARITHMATIC.lt.%d.IF_TRUE)\n", COMP_LT_COUNT); // (ARITHMATIC.lt.%d.IF_TRUE)
    write_cpu(output_file, "D", "-1"); // D=-1
    write_template(output_file, "(ARITHMATIC.lt.%d.END)\n", COMP_LT_COUNT); // (ARITHMATIC.lt.%d.END)
    push_top(output_file);
    COMP_LT_COUNT++;
}

void bit_and(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write_cpu(output_file, "M", "M&D"); // D=M&D
    increment_sp(output_file);
}

void bit_or(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write_cpu(output_file, "M", "M|D"); // D=M|D
    increment_sp(output_file);
}

void bit_not(FILE* output_file) {
    decrement_sp(output_file);
    write_cpu(output_file, "M", "!M"); // D=!M
    increment_sp(output_file);
}
//////////////////////// MATH OPERTATIONS /////////////////////
//////////////////////// PRIM OPERTATIONS /////////////////////
void pop_top(FILE* output_file) {
    decrement_sp(output_file);
    write_cpu(output_file, "D", "M"); // D=M
    return;
}

// Pushes data reg to top of stack;
void push_top(FILE* output_file) {
    write_address(output_file, "SP"); // @SP
    write_cpu(output_file, "A", "M"); // A=M
    write_cpu(output_file, "M", "D"); // M=D
    increment_sp(output_file);
    return;
}

// increments the stack pointer and points address reg at virtual address
void increment_sp(FILE* output_file) {
    write_address(output_file, "SP"); // @SP
    write_cpu(output_file, "AM", "M+1"); // A=M+1
    return;
}

// decrements the stack pointer and points address reg at virtual address.
void decrement_sp(FILE* output_file) {
    write_address(output_file, "SP"); // @SP
    write_cpu(output_file, "AM", "M-1"); // A=M-1
    return;
}

void write_comment(FILE* output_file, TOKEN token) {
    write_template(output_file, "// %s %s %s\n", token.type, token.arg1, token.arg2);
    return;
}

void write_cpu(FILE* output_file, char* destination, char* source) {
    write_template(output_file, "%s=%s\n", destination, source);
}

void write_address(FILE* output_file, char* destination) {
    write_template(output_file, "@%s\n", destination);
}

void write_jump(FILE* output_file, char* destination, char* jump) {
    write_template(output_file, "%s;%s\n", destination, jump);
}

void write_template(FILE *output_file, char *template, ...) {
    va_list args;
    va_start(args, template);
    char buffer[128]; 
    vsprintf(buffer, template, args);
    write(output_file, buffer);
    va_end(args);
}

void write(FILE* output_file, char* string) {
    fwrite(string, sizeof(char), strlen(string), output_file);
    return;
}
//////////////////////// PRIM OPERTATIONS /////////////////////