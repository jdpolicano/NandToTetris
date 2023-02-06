#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
#include <math.h>
#include "parser.h"

static int COMP_EQ_COUNT = 1;

// might want to ocnsider returning an integer to indicate success or failure, but for now strategy is to exit program an print error if we hit a bump.
void translate(TOKEN_ARRAY* tokens, FILE* output_file);
void push_top(FILE* output_file);
void pop_top(FILE* output_file);
void add(FILE* output_file);
void subtract(FILE* output_file);
void negate(FILE* output_file);
void equality(FILE* output_file);
void write_template_string(char* template, int count, FILE* output_file);
void increment_sp(FILE* output_file);
void decrement_sp(FILE* output_file);
void write_comment(TOKEN token, FILE* output_file);
void write(char* string, FILE* output_file);

void translate(TOKEN_ARRAY* tokens, FILE* output_file) {
    
    for (int i = 0; i < tokens->length; i++) {
        TOKEN curr = tokens->tokens[i];
        if (strcmp(curr.type, C_MATH) == 0) {
            if (strcmp(curr.arg1, "add") == 0) {
                write_comment(curr, output_file);
                add(output_file); 
            } else if (strcmp(curr.arg1, "sub") == 0) {
                write_comment(curr, output_file);
                subtract(output_file);
            } else if (strcmp(curr.arg1, "neg") == 0) {
                write_comment(curr, output_file);
                negate(output_file);
            } else if (strcmp(curr.arg1, "eq") == 0) {
                write_comment(curr, output_file);
                equality(output_file);
            }
        }
    }
}


// more complex operations that combine prims;
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
    pop_top(output_file); 
    write("@0\n", output_file);
    write("D=A-D\n", output_file);
    push_top(output_file);
}


void equality(FILE* output_file) {
    pop_top(output_file);
    write("A=A-1\n", output_file); // slight optimization.
    write("D=D-M\n", output_file); // data should now be zero or something else;
    write_template_string("@comp.eq.%d_true\n", COMP_EQ_COUNT, output_file);
    write("D;JEQ\n", output_file);
    write("D=0\n", output_file);
    write_template_string("@comp.eq.%d_end\n", COMP_EQ_COUNT, output_file);
    write("0;JMP\n", output_file);
    write_template_string("(comp.eq.%d_true)\n", COMP_EQ_COUNT, output_file);
    write("D=-1\n", output_file);
    write_template_string("(comp.eq.%d_end)\n", COMP_EQ_COUNT, output_file);
    push_top(output_file);
    COMP_EQ_COUNT++;
}

void write_template_string(char* template, int count, FILE* output_file) {
    int int_length = (int)(sizeof(char) * (ceil(log10(count)) + 1));
    char* template_buffer = malloc(sizeof(char) * (strlen(template) + int_length));
    sprintf(template_buffer, template, count);
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