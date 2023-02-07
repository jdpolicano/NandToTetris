#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
#include <math.h>
#include "parser.h"

static int COMP_EQ_COUNT = 1;
static int COMP_GT_COUNT = 1;
static int COMP_LT_COUNT = 1;

// might want to ocnsider returning an integer to indicate success or failure, but for now strategy is to exit program an print error if we hit a bump.
void translate(TOKEN_ARRAY* tokens, FILE* output_file);

// Push
int is_push(char* type);
void route_push(TOKEN push_token, FILE* output_file);
void push_constant(char* value, FILE* output_file);
void push_virtual(char* segement, char* index, FILE* output_file);
// Push identifiers
int is_push_constant(char* segment);
int is_push_virtual(char* segement);
// MAths
int is_math(char* type);
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
// prims
void write_template_string_int(char* template, int count, FILE* output_file);
void write_template_string_chars(char* template, char* index, FILE* output_file);
void push_top(FILE* output_file);
void pop_top(FILE* output_file);
void increment_sp(FILE* output_file);
void decrement_sp(FILE* output_file);
void write_comment(TOKEN token, FILE* output_file);
void write(char* string, FILE* output_file);

void translate(TOKEN_ARRAY* tokens, FILE* output_file) {
    for (int i = 0; i < tokens->length; i++) {
        TOKEN curr = tokens->tokens[i];
        if (is_math(curr.type)) {
            route_math(curr, output_file);
        } else if (is_push(curr.type)) {
            route_push(curr, output_file);
        }
    }
}

// PUSH & POP
int is_push(char* type) {
    return strcmp(type, C_PUSH) == 0;
}

void route_push(TOKEN push_token, FILE* output_file) {
    char* segment = push_token.arg1;
    char* index = push_token.arg2;
    int lineNum = push_token.lineNum;

    if (segment == NULL || index == NULL) {
        printf("Error in token at line %i :: expected char* but received NULL\n", lineNum);
        exit(1); 
    }

    write_comment(push_token, output_file);
    
    if (is_push_constant(segment)) {
        push_constant(index, output_file);
    } else if (is_push_virtual(segment)) {
        push_virtual(segment, index, output_file);
    }
}

// Push operations
void push_constant(char* value, FILE* output_file) {
    write_template_string_chars("@%s\n", value, output_file);
    write("D=A\n", output_file);
    push_top(output_file);
}

void push_virtual(char* segement, char* index, FILE* output_file) {
    char* target;

    if (strcmp(segement, "this") == 0) target = "THIS";
    if (strcmp(segement, "that") == 0) target = "THAT";
    if (strcmp(segement, "local") == 0) target = "LCL";
    if (strcmp(segement, "argument") == 0) target = "ARG";

    write_template_string_chars("@%s\n", index, output_file);
    write("D=A\n", output_file);
    write_template_string_chars("@%s\n", target, output_file);
    write("A=D+M\n", output_file); 
    write("D=M\n", output_file);
    push_top(output_file);
}


// Push identifiers
int is_push_constant(char* segment) {
    return strcmp("constant", segment) == 0;
}

int is_push_virtual(char* segement) {
    char* virtual_addresses[] = { "this", "that", "argument", "local" };
    int num_addresses = 4;

    for (int i = 0; i < num_addresses; i++) {
        if (strcmp(virtual_addresses[i], segement) == 0) {
            return 1;
        }
    }
    
    return 0; 
}

// MATH OPERTATIONS
int is_math(char* type) {
    return strcmp(type, C_MATH) == 0;
}

void route_math(TOKEN math_token, FILE* output_file) {
    char* operation = math_token.arg1;

    if (strcmp(operation, "add") == 0) {
        write_comment(math_token, output_file);
        add(output_file); 
    } else if (strcmp(operation, "sub") == 0) {
        write_comment(math_token, output_file);
        subtract(output_file);
    } else if (strcmp(operation, "neg") == 0) {
        write_comment(math_token, output_file);
        negate(output_file);
    } else if (strcmp(operation, "eq") == 0) {
        write_comment(math_token, output_file);
        equality(output_file);
    } else if (strcmp(operation, "gt") == 0) {
        write_comment(math_token, output_file);
        greater_than(output_file);
    } else if (strcmp(operation, "lt") == 0) {
        write_comment(math_token, output_file);
        less_than(output_file);
    } else if (strcmp(operation, "and") == 0) {
        write_comment(math_token, output_file);
        bit_and(output_file);
    } else if (strcmp(operation, "or") == 0) {
        write_comment(math_token, output_file);
        bit_or(output_file);
    } else if (strcmp(operation, "not") == 0) {
        write_comment(math_token, output_file);
        bit_not(output_file);
    }
}

void add(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write("M=M+D\n", output_file); 
    increment_sp(output_file);
}


void subtract(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write("M=M-D\n", output_file); // M=M-D
    increment_sp(output_file);
}

void negate(FILE* output_file) {
    decrement_sp(output_file); 
    write("D=-M\n", output_file);
    push_top(output_file);
}


void equality(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file); 
    write("D=M-D\n", output_file); // data should now be zero or something else;
    write_template_string_int("@ARITHMATIC.eq.%d.IF_TRUE\n", COMP_EQ_COUNT, output_file);
    write("D;JEQ\n", output_file);
    write("D=0\n", output_file);
    write_template_string_int("@ARITHMATIC.eq.%d.END\n", COMP_EQ_COUNT, output_file);
    write("0;JMP\n", output_file);
    write_template_string_int("(ARITHMATIC.eq.%d.IF_TRUE)\n", COMP_EQ_COUNT, output_file);
    write("D=-1\n", output_file);
    write_template_string_int("(ARITHMATIC.eq.%d.END)\n", COMP_EQ_COUNT, output_file);
    push_top(output_file);
    COMP_EQ_COUNT++;
}

void greater_than(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write("D=M-D\n", output_file); 
    write_template_string_int("@ARITHMATIC.gt.%d.IF_TRUE\n", COMP_GT_COUNT, output_file);
    write("D;JGT\n", output_file);
    write("D=0\n", output_file);
    write_template_string_int("@ARITHMATIC.gt.%d.END\n", COMP_GT_COUNT, output_file);
    write("0;JMP\n", output_file);
    write_template_string_int("(ARITHMATIC.gt.%d.IF_TRUE)\n", COMP_GT_COUNT, output_file);
    write("D=-1\n", output_file);
    write_template_string_int("(ARITHMATIC.gt.%d.END)\n", COMP_GT_COUNT, output_file);
    push_top(output_file);
    COMP_GT_COUNT++;
}

void less_than(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write("D=M-D\n", output_file); 
    write_template_string_int("@ARITHMATIC.lt.%d.IF_TRUE\n", COMP_LT_COUNT, output_file);
    write("D;JLT\n", output_file);
    write("D=0\n", output_file);
    write_template_string_int("@ARITHMATIC.lt.%d.END\n", COMP_LT_COUNT, output_file);
    write("0;JMP\n", output_file);
    write_template_string_int("(ARITHMATIC.lt.%d.IF_TRUE)\n", COMP_LT_COUNT, output_file);
    write("D=-1\n", output_file);
    write_template_string_int("(ARITHMATIC.lt.%d.END)\n", COMP_LT_COUNT, output_file);
    push_top(output_file);
    COMP_LT_COUNT++;
}

void bit_and(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write("D=D&M\n", output_file); // shouldn't matter which order its in. 
    push_top(output_file);
}

void bit_or(FILE* output_file) {
    pop_top(output_file);
    decrement_sp(output_file);
    write("D=D|M\n", output_file); // shouldn't matter which order its in. 
    push_top(output_file);
}

void bit_not(FILE* output_file) {
    decrement_sp(output_file);
    write("D=!M\n", output_file);
    push_top(output_file);
}

void write_template_string_int(char* template, int input, FILE* output_file) {
    int int_length = (int)(sizeof(char) * (ceil(log10(input)) + 1));
    char* template_buffer = malloc(sizeof(char) * (strlen(template) + int_length));
    sprintf(template_buffer, template, input);
    write(template_buffer, output_file);
    free(template_buffer);
}

void write_template_string_chars(char* template, char* input, FILE* output_file) {
    int input_length = strlen(input);
    int template_length = strlen(template);
    char* template_buffer = malloc(sizeof(char) * (template_length + input_length + 1));
    sprintf(template_buffer, template, input);
    write(template_buffer, output_file);
    free(template_buffer);
}

// Primitive Translations that can combine to produce simple commands;
// Pops data off of top of stack and stores in data register;
void pop_top(FILE* output_file) {
    decrement_sp(output_file);
    write("D=M\n", output_file); // D=M
    return;
}

// Pushes data reg to top of stack;
void push_top(FILE* output_file) {
    write("@SP\n", output_file); // @SP
    write("A=M\n", output_file); // A=M
    write("M=D\n", output_file); // M=D
    increment_sp(output_file);
    return;
}

// increments the stack pointer and points address reg at virtual address
void increment_sp(FILE* output_file) {
    write("@SP\n", output_file); // @SP
    write("AM=M+1\n", output_file); // M=M+1
    return;
}

// decrements the stack pointer and points address reg at virtual address.
void decrement_sp(FILE* output_file) {
    write("@SP\n", output_file); // @SP
    write("AM=M-1\n", output_file); // M=M-1
    return;
}

void write_comment(TOKEN token, FILE* output_file) {
    write("// ", output_file);
    if (token.type != NULL) write(token.type, output_file);
    write(" ", output_file);
    if (token.arg1 != NULL) write(token.arg1, output_file);
    write(" ", output_file);
    if (token.arg2 != NULL) write(token.arg2, output_file);
    write("\n", output_file);
    return;
}

void write(char* string, FILE* output_file) {
    fwrite(string, sizeof(char), strlen(string), output_file);
    return;
}