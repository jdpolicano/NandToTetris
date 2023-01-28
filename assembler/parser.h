#ifndef PARSER_H
#define PARSER_H
#define A_TYPE "A_TYPE"
#define C_TYPE_ASSIGN "C_TYPE_ASSIGN"
#define C_TYPE_JUMP "C_TYPE_JUMP"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

typedef struct {
  char* key;
  int value;
} SYMBOL;

typedef struct {
  int size;
  SYMBOL* data;
} SYMBOL_ARRAY;

typedef struct {
  char* type; // universal
  char* dest; // only for C_TYPE
  char* comp; // only for C_TYPE
  char* jump; // only for C_TYPE
  int data; // only for A_TYPE -> L_TYPES essentially become A_TYPES; 
} TOKEN;

typedef struct {
  int size;
  TOKEN* data;
} TOKEN_ARRAY;  

TOKEN_ARRAY* parse(FILE* source);
void free_parsed_tokens(TOKEN_ARRAY* token_array);

#endif