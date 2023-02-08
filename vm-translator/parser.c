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

// to be shared downstream; 
TOKEN_ARRAY* PARSED_TOKENS;

// Function definitions (prototypes) that are shared
TOKEN_ARRAY* parse(FILE* vm_code);
void free_token_array();

static void parse_line(FILE* vm_code);
static char** parse_arguments(char* line, char** container, int container_size); 
static void tokenize_statement(char* type, char* arg1, char* arg2);
static char* read_line(FILE* vm_code);
static void skip_carriage_return(FILE* vm_code);
static int is_push(char** arguments);
static int is_pop(char** arguments);
static int is_math(char** arguments);
static int is_valid_line(char** arguments);
static void put_token(TOKEN token);

// for later processing steps and error handling...should print line num etc
static int validate_type(char* line, int line_size);
static int validate_segment(char* line, int line_size);
static int validate_index(char* line, int line_size);
static void init_token_array(); 




TOKEN_ARRAY* parse(FILE* vm_code) {
    init_token_array();

    while(!feof(vm_code)) {
        parse_line(vm_code);
    }

    PARSED_TOKENS->length = ARRAY_LENGTH;
    return PARSED_TOKENS;
};

// Reads in a line of text, and parses it, adding it to PARSED TOKEN if valid;  
static void parse_line(FILE* vm_code) {
    char* next_line = read_line(vm_code);
    char* arguments[MAX_ARGUMENTS];
    // side effect should fill our container.
    parse_arguments(next_line, arguments, MAX_ARGUMENTS);

    if (!is_valid_line(arguments)) return;

    // route to the appropriate handler; 
    if (is_math(arguments)) {
        tokenize_statement(C_MATH, arguments[0], arguments[1]);
        return;
    }

    if (is_push(arguments)){
        tokenize_statement(C_PUSH, arguments[1], arguments[2]);
        return;
    }

    if (is_pop(arguments)) {
        tokenize_statement(C_POP, arguments[1], arguments[2]);
        return;
    }
    // to-do write functions for other statement types...


    // handle syntax errors
    printf("Error parsing with args: %s, %s, %s :: at line %i\n", arguments[0], arguments[1], arguments[2], LINE_COUNT);
    // exit should free any memory allocated?
    exit(1);
    return;
}


static char** parse_arguments(char* line, char** container, int container_size) {
    char* delimiters = " ";
    char* toke = strtok(line, delimiters);
   // fill arguments with null;
    for (int i = 0; i < container_size; i++) {
        container[i] = NULL;
    }
    
    // continue copying tokens into container until we reach null;
    for (int i = 0; i < container_size && toke != NULL; i++) {
        char* arg = malloc(sizeof(char) * strlen(toke));
        if (arg == NULL) {
            printf("Error parsing line %s, at line %i\n", line, LINE_COUNT);
            free(toke);
            free(arg);
            exit(1);
        }
        strcpy(arg, toke);
        container[i] = arg;
        toke = strtok(NULL, delimiters);
    }

    return container;
}

static void tokenize_statement(char* type, char* arg1, char* arg2) {
    TOKEN new_entry;
    new_entry.lineNum = LINE_COUNT;
    new_entry.type = type;
    new_entry.arg1 = arg1;
    new_entry.arg2 = arg2;
    put_token(new_entry);
    return;
}

static char* read_line(FILE* vm_code) {
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
    // I think I need this because there is both a carriage return and a newline at the end of some of the lines...
    // looks ahead for a carriage return and moves file pointer if found after newline.
    skip_carriage_return(vm_code);

    return text;
}

static void skip_carriage_return(FILE* vm_code) {
    char buffer = fgetc(vm_code);
    while (buffer == '\r') {
        buffer = fgetc(vm_code);
    }
    fseek(vm_code, -1, SEEK_SET);
    return;
}

static int is_math(char** arguments) {
    if (arguments[0] == NULL) return 0;

    for (int i = 0; i < NUM_MATH_OPS; i++) {
        if (strcmp(arguments[0], MATH_OPS[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

static int is_push(char** arguments) {
    if (arguments[0] == NULL) return 0;

    return strcmp(arguments[0], PUSH) == 0; 
}

static int is_pop(char** arguments) {
    if (arguments[0] == NULL) return 0;

    return strcmp(arguments[0], POP) == 0; 
}

static int is_valid_line(char** arguments) {
    // is not NULL, empty, or a comment; 
    return arguments[0] != NULL && strlen(arguments[0]) && arguments[0][0] != '/';
}

static void init_token_array() {
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

void free_token_array() {
    for (int i = 0; i < ARRAY_LENGTH; i++) {
        TOKEN curr = PARSED_TOKENS->tokens[i];
        free(curr.arg2);
        free(curr.arg2);
    }

    free(PARSED_TOKENS->tokens);
    free(PARSED_TOKENS);
}

static void put_token(TOKEN token) {
    if (ARRAY_LENGTH == ARRAY_MAX_SIZE) {
        ARRAY_MAX_SIZE *= 2;
        TOKEN* tmp = realloc(PARSED_TOKENS->tokens, sizeof(TOKEN) * ARRAY_MAX_SIZE);
        if (tmp == NULL) {
            printf("Unable to alloc mem for new token at line %i\n", LINE_COUNT);
            free_token_array();
            exit(1);
        }
        PARSED_TOKENS->tokens = tmp;
    }

    PARSED_TOKENS->tokens[ARRAY_LENGTH] = token;
    ARRAY_LENGTH++;
    return;
}




