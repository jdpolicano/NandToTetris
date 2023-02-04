#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
// operation categories
#define C_MATH "C_MATH"
#define C_PUSH "C_PUSH"
#define C_POP "C_POP"
// to be handled later...
#define C_LABEL "C_LABEL"
#define C_GOTO "C_GOTO"
#define C_FUNC "C_FUNC"
#define C_RETURN "C_RETURN"
#define C_CALL "C_CALL"
// so we can iterate

typedef struct {
    int lineNum; 
    char* type;
    char* arg1;
    char* arg2;
} TOKEN;

typedef struct {
    int length;
    TOKEN* tokens;
} TOKEN_ARRAY;

static int NUM_MATH_OPS = 9;
static char* MATH_OPS[] = { "add", "sub", "neg", "eq", "gt", "lt", "and", "or", "not" };
static char* PUSH = "push";
static char* POP = "pop";
static int MAX_ARGUMENTS = 3;
static int LINE_COUNT = 0;
static int ARRAY_MAX_SIZE = 1000;
static int ARRAY_LENGTH = 0;
TOKEN_ARRAY* PARSED_TOKENS;

TOKEN_ARRAY* parse(FILE* vm_code);
void parse_line(FILE* vm_code);
char* read_line(FILE* vm_code);
int is_push(char* line);
int is_pop(char* line);
int is_math(char* line);
int is_valid_line(char* line);
char* alloc_arg(char* line);
int validate_type(char* line, int line_size);
int validate_segment(char* line, int line_size);
int validate_index(char* line, int line_size);
char* get_arg1(char* line, int line_size);
char* get_arg2(char* line, int line_size);
void init_token_array(); 


TOKEN_ARRAY* parse(FILE* vm_code) {
    init_token_array();

    while(!feof(vm_code)) {
        parse_line(vm_code);
    }

    return PARSED_TOKENS;
};

// Reads in a line of text, and parses it, adding it to PARSED TOKEN if valid;  
void parse_line(FILE* vm_code) {
    char* next_line = read_line(vm_code);
    char* delimiters = " ";
    char* toke = strtok(next_line, delimiters);
    char** arguments = malloc(sizeof(char*) * MAX_ARGUMENTS); // three arguments max per command; 

    if (!is_valid_line(toke)) return;

    // fill arguments with null;
    for (int i = 0; i < MAX_ARGUMENTS; i++) {
        arguments[i] = NULL;
    }
    
    // continue copying tokens into arguments until we reach null;
    for (int i = 0; i < MAX_ARGUMENTS && toke != NULL; i++) {
        char* arg = malloc(sizeof(char) * strlen(toke));
        if (arg == NULL) {
            printf("Error parsing line %s, at line %i\n", next_line, LINE_COUNT);
            free(toke);
            free(arg);
            exit(1);
        }
        strcpy(arg, toke);
        arguments[i] = arg;
        toke = strtok(NULL, delimiters);
    }

    if (is_math(arguments)) {
        tokenize_math_statement(arguments);
        return;
    }

    if (is_push(arguments)){
        tokenize_push_statement(arguments);
        return;
    }

    if (is_pop(arguments)) {
        tokenize_pop_statement(arguments);
    }

    // to-do write functions for other statement types...
    free(toke); 
    return;
}

char* read_line(FILE* vm_code) {
    LINE_COUNT++;
    int max_length = 10;
    int length = 0; 
    char* text = malloc(max_length);
    char buffer = fgetc(vm_code);

    while(buffer != EOF && buffer != '\n' && buffer != '\r') {
        // expand array if necessary.
        if (length == max_length - 1) {
            max_length *= 2;
            char* tmp = realloc(text, sizeof(char) * max_length);
            if (tmp == NULL) {
                printf("Unable to allocate space for addition characters\n");
                free(text);
                exit(1);
            }
            text = tmp;
        }

        text[length] = buffer;
        length++;
        buffer = fgetc(vm_code);
    }

    text[length] = '\0';
    return text;
}

int is_math(char* line) {
    for (int i = 0; i < NUM_MATH_OPS; i++) {
        if (strcmp(line, MATH_OPS[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

int is_push(char* line) {
    return strcmp(line, PUSH) == 0; 
}

int is_pop(char* line) {
    return strcmp(line, POP) == 0; 
}

int is_valid_line(char* line) {
    // is not NULL, empty, or a comment; 
    return line != NULL && strlen(line) && line[0] != '/';
}

char* alloc_arg(char* line) {
    char* arg = malloc(strlen(line));

    if (arg == NULL || line == NULL) {
        printf("Error allocating memory for argument '%s' at line %i\n", line, LINE_COUNT);
        exit(0);
    }

    strcpy(arg, line);

    return line;
}

void init_token_array() {
    TOKEN_ARRAY* tmp_parsed = malloc(sizeof(TOKEN_ARRAY));
    TOKEN* tmp_data = malloc(sizeof(TOKEN*) * ARRAY_MAX_SIZE);

    if (tmp_parsed == NULL || tmp_data == NULL) {
        printf("Error in alloc mem for init token array\n");
        free(tmp_parsed);
        free(tmp_data);
        exit(1);
    }

    PARSED_TOKENS = tmp_parsed;
    PARSED_TOKENS->length = 0;
    PARSED_TOKENS->tokens = tmp_data;
    return; 
}




