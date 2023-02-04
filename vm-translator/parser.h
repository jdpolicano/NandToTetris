#ifndef VM_PARSER_H
#define VM_PARSER_H

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>

typedef struct {
    char* type;
    char* arg1;
    char* arg2;
} TOKEN;

typedef struct {
    int length;
    TOKEN* tokens;
} TOKEN_ARRAY;

TOKEN_ARRAY* parse(FILE* vm_code);

#endif