#ifndef VM_PARSER_H
#define VM_PARSER_H

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>

#define C_MATH "C_MATH"
#define C_PUSH "C_PUSH"
#define C_POP "C_POP"
// to be handled later...
#define C_LABEL "C_LABEL"
#define C_GOTO "C_GOTO"
#define C_IF_GOTO "C_IF_GOTO"
#define C_FUNC "C_FUNC"
#define C_RETURN "C_RETURN"
#define C_CALL "C_CALL"

typedef struct {
    int lineNum;
    char* type;
    char* arg1;
    char* arg2;
} TOKEN;

typedef struct {
    int length;
    char* file_name;
    TOKEN* tokens;
} TOKEN_ARRAY;

TOKEN_ARRAY* parse(FILE* vm_code, char* file_name);
void free_token_array(TOKEN_ARRAY* tokens_array);

#endif