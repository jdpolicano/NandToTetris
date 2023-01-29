#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

// Definitions.
// End of line
#define EOL '\n'
#define RETURN '\r'
#define COMMENT '/'
#define SPACE ' '
#define OPEN_PAREN '('
#define CLOSE_PAREN ')'
#define AREG '@'
#define JUMP ';'
#define ASSIGN '='
#define A_TYPE "A_TYPE"
#define C_TYPE_ASSIGN "C_TYPE_ASSIGN"
#define C_TYPE_JUMP "C_TYPE_JUMP"
#define MAX_DESTINATION_SIZE 5
#define MAX_COMPARISON_SIZE 4
#define MAX_JUMP_SIZE 5

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
void register_labels(FILE* source);
char* parse_label(char* read_line, int line_length);
void parse_tokens(FILE* source);
TOKEN parse_a_type(char* line, int line_size);
TOKEN parse_c_type_assignment(char* line, int line_size);
TOKEN parse_c_type_jump(char* line, int line_size);
int is_number(char* line, int line_size);
int is_jump(char* line, int line_size);
int is_assignment(char* line, int line_size);
int is_AREG(char* line, int line_size);
int is_label(char* line, int line_size); 
char* read_line(FILE* source);
int is_valid(char ch);
char seek_start(FILE* source);
char skip_newlines (FILE* source);
char skip_space(FILE* source);
char skip_comment(FILE* source);
int get_symbol(char* key);
void put_symbol(char* key, int value);
void put_token(TOKEN new_entry);
void init_sym_table(void);
void free_sym_table(); 
void init_parsed_tokens(void);
void free_parsed_tokens(TOKEN_ARRAY* token_array);


// The predefined symbols.
static int MAX_TABLE_SIZE = 10;
static int MAX_TOKEN_SIZE = 10; 

// Used to hanlde symbolic jumps for ROM labels. i.e., (something)...
static int LINE_COUNT = 0; 
// Denotes the next block of memory to store a symbolic variable at. Starts at 16; 
static int MEMORY_ADDRESS = 16;

SYMBOL_ARRAY* SYM_TABLE;
TOKEN_ARRAY* PARSED_TOKENS;

TOKEN_ARRAY* parse(FILE* source) {
  init_sym_table();
  init_parsed_tokens();
  register_labels(source);
  fseek(source, 0, SEEK_SET);
  parse_tokens(source);
  free_sym_table();
  return PARSED_TOKENS;
}

/////////////////////////// CORE FUNCTIONS /////////////////////////////////////

void register_labels(FILE* source) {
  printf("Begin label search\n");

  char* text = read_line(source);
  int length = strlen(text);

  while(length) {
    if (text[0] == OPEN_PAREN) {
      char* label = parse_label(text, length);
      put_symbol(label, LINE_COUNT);
      text = read_line(source);
      length = strlen(text);
      continue;
    }
 
    LINE_COUNT++;
    text = read_line(source);
    length = strlen(text);
  }

  printf("End Search. Identified labels\n");
  return;
}

char* parse_label(char* read_line, int line_length) {
  char* label = malloc(sizeof(char) * line_length);
  int label_len = 0;

  if (label == NULL) {
    printf("Unable to alloc mem for label at line %i\n", LINE_COUNT);
    free(label);
    exit(1);
  }

  // start at 1 to skip open parens
  for (int i = 1; read_line[i] != CLOSE_PAREN; i++) {
    label[label_len] = read_line[i];
    label_len++;
  }

  label[label_len] = '\0';

  return label;
}

void parse_tokens(FILE* source) {
  printf("Begin Tokenizing\n");
  char* text = read_line(source);
  int length = strlen(text);

  while (length) {
    if (is_AREG(text, length)) {
      // printf("Is areg %s\n", text);
      TOKEN new_entry = parse_a_type(text, length);
      put_token(new_entry);
      text = read_line(source);
      length = strlen(text);
    }

    else if (is_assignment(text, length)) {
      // printf("Is c type assign %s\n", text);
      TOKEN new_entry = parse_c_type_assignment(text, length);
      put_token(new_entry);
      text = read_line(source);
      length = strlen(text);
    }

    else if (is_jump(text, length)) {
      // printf("Is c type jump %s\n", text);
      TOKEN new_entry = parse_c_type_jump(text, length);
      put_token(new_entry);
      text = read_line(source);
      length = strlen(text);
    }

    else if (is_label(text, length)) {
      // printf("Is label.\n");
      text = read_line(source);
      length = strlen(text);
    }

    else {
      // print error because soemthing fucked up...
      printf("Opps at line %s\n", text);
      return;
    }
  }
  printf("End Tokenizing\n");
}

TOKEN parse_a_type(char* line, int line_size) {
  char* substring = malloc(line_size);

  if (substring == NULL) {
    printf("Unable to allocate for substring in line %s\n", line);
    free(substring);
    exit(1);
  }

  TOKEN result;
  // again string literal will not need to be freed. 
  result.type = A_TYPE;

  for(int i = 1; i < line_size; i++) {
    substring[i - 1] = line[i];
  }

  substring[line_size - 1] = '\0';

  if (is_number(substring, line_size - 1)) {
    result.data = atoi(substring);
    free(substring);
    return result;
  } else {
    int sym_value = get_symbol(substring);
    // not found - first declaration;
    if (sym_value < 0) {
      put_symbol(substring, MEMORY_ADDRESS);
      result.data = MEMORY_ADDRESS;
      MEMORY_ADDRESS++;
      return result;
    } else {
      result.data = sym_value;
      free(substring);
      return result; 
    }
  }
}

TOKEN parse_c_type_assignment(char* line, int line_size) {
  char* destination = malloc(MAX_DESTINATION_SIZE);
  int dest_size = 0;

  char* comparison = malloc(MAX_COMPARISON_SIZE); 
  int comp_size = 0;

  char* jump = malloc(MAX_JUMP_SIZE);
  strcpy(jump, "null");


  if (destination == NULL || comparison == NULL || jump == NULL) {
    printf("Failed to parse c type assign with line %s\n", line);
    free(destination);
    free(comparison);
    free(jump);
    exit(1);
  }

  TOKEN new_entry;
  new_entry.type = C_TYPE_ASSIGN;
  new_entry.dest = destination;
  new_entry.comp = comparison;
  new_entry.jump = jump;
  new_entry.data = -1; 

  // break up the destination and comparison;

  for (int i = 0; line[i] != ASSIGN; i++) {
    destination[dest_size] = line[i];
    dest_size++;
  }
  destination[dest_size] = '\0';
  dest_size++;

  // begin where the last one left off an go to end;
  for (int j = dest_size; j < line_size; j++) {
    comparison[comp_size] = line[j];
    comp_size++;
  }
  comparison[comp_size] = '\0';

  return new_entry; 
}

TOKEN parse_c_type_jump(char* line, int line_size) {
  char* destination = malloc(MAX_DESTINATION_SIZE);
  strcpy(destination, "null");

  char* comparison = malloc(MAX_COMPARISON_SIZE); 
  int comp_size = 0;

  char* jump = malloc(MAX_JUMP_SIZE);
  int jump_size = 0; 


  if (destination == NULL || comparison == NULL || jump == NULL) {
    printf("Failed to parse c type jump with line %s\n", line);
    free(destination);
    free(comparison);
    free(jump);
    exit(1);
  }

  TOKEN new_entry;
  new_entry.type = C_TYPE_JUMP;
  new_entry.dest = destination;
  new_entry.comp = comparison;
  new_entry.jump = jump;
  new_entry.data = -1;

  for (int i = 0; line[i] != JUMP; i++) {
    comparison[comp_size] = line[i];
    comp_size++;
  } 
  comparison[comp_size] = '\0';
  comp_size++;

  for (int j = comp_size; j < line_size; j++) {
    jump[jump_size] = line[j];
    jump_size++;
  }
  jump[jump_size] = '\0';


  return new_entry;
}

// determines if a text string is a number or not. returns 0 if false and 1 if true; 
int is_number(char* line, int line_size) {
  for (int i = 0; i < line_size; i++) {
    if (!isdigit(line[i])) return 0;
  }

  return 1;
}

int is_assignment(char* line, int line_size) {
  for (int i = 0; i < line_size; i++) {
    if (line[i] == ASSIGN) return 1;
  }

  return 0;
}

int is_jump(char* line, int line_size) {
  for (int i = 0; i < line_size; i++) {
    if (line[i] == JUMP) return 1;
  }

  return 0;
}

int is_AREG(char* line, int line_size) {
  return line[0] == AREG;
}

int is_label(char* line, int line_size) {
  return line[0] == OPEN_PAREN;
}
/////////////////////////// CORE FUNCTIONS /////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////
/////////////////////////// HELPERS /////////////////////////////////////

// Reads a line of valid text into memory and returns a c-string, empty string indicates EOF. EOF could be an error or end of file. Who knows... 
char* read_line(FILE* source) {
  // make sure we haven't re
  // characters to malloc at first;
  int read_max = 10; 
  int read_length = 0;
  char* text = malloc(sizeof(char) * read_max);

  // Seek the first valid character; 
  char char_buffer = seek_start(source);

  // check that we have not reached eof
  if (char_buffer == EOF) {
    if (feof(source)) {
      return "\0"; // zero length string indicates nothing left to read;
    } else {
      printf("Error occurred in file read operation\n");
      // to-do check error status
      free(text);
      exit(1); 
    }
  }

  // prcocess the next valid section of text;
  while (is_valid(char_buffer)) {
    // resize our buffer here if needed;
    if (read_length == read_max) {
      read_max *= 2;
      char* tmp = realloc(text, sizeof(char) * read_max);
      if (tmp == NULL) {
        printf("unable to allocate more space for text chunck\n");
        free(text);
        exit(1);
      }
      text = tmp;
    }

    text[read_length] = char_buffer;
    read_length++;
    char_buffer = fgetc(source);
  }

  // add null terminator;
  text[read_length] = '\0';
  return text;
}

// Finds the beginning of the next valid line. Skips comments, spaces, newlines, returns, etc...
char seek_start(FILE* source) {
  char buffer = fgetc(source);

  while (buffer != EOF) {
    if (buffer == COMMENT) {
      buffer = skip_comment(source);
      continue;
    }

    if (buffer == SPACE) {
      buffer = skip_space(source);
      continue;
    }

    if (buffer == EOL || buffer == RETURN) {
      buffer = skip_newlines(source);
      continue;
    }

    break;
  } 

  return buffer;
}

int is_valid(char ch) {
  return ch != EOL && ch != RETURN && ch != SPACE && ch != EOF;
}

// will read until the next non eol character.
char skip_newlines (FILE* source) {
  char ch = fgetc(source);

  while (ch == EOL || ch == RETURN) {
    ch = fgetc(source);
  }

  return ch;
}

// Will read the next valid non space character;
char skip_space(FILE* source) {
  char ch;

  while ((ch = fgetc(source)) == SPACE);

  return ch;
}

// This will advance the file stream to newline, return, or EOF. 
char skip_comment(FILE* source) {
  char ch = fgetc(source);

  while(ch != EOL && ch != RETURN && ch != EOF) {
    ch = fgetc(source);
  }

  return ch;
}

/////////////////////////// HELPERS /////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
/////////////////////////// INITALIZERS /////////////////////////////////////
// Initializes the symbol table with default values. 
void init_sym_table(void) {
  // initialize R reg values
  SYM_TABLE = malloc(sizeof(SYMBOL_ARRAY));
  SYM_TABLE->data = malloc(sizeof(SYMBOL) * MAX_TABLE_SIZE);

  if (SYM_TABLE == NULL || SYM_TABLE->data == NULL) {
    printf("Unable to intialize sym_table\n");
    free(SYM_TABLE); 
    exit(1); 
  }

  SYM_TABLE->size = 0;

  put_symbol("R0", 0);
  put_symbol("R1", 1);
  put_symbol("R2", 2);
  put_symbol("R3", 3);
  put_symbol("R4", 4);
  put_symbol("R5", 5);
  put_symbol("R6", 6);
  put_symbol("R7", 7);
  put_symbol("R8", 8);
  put_symbol("R9", 9);
  put_symbol("R10", 10);
  put_symbol("R11", 11);
  put_symbol("R12", 12);
  put_symbol("R13", 13);
  put_symbol("R14", 14);
  put_symbol("R15", 15);
  put_symbol("SP", 0);
  put_symbol("LCL", 1);
  put_symbol("ARG", 2);
  put_symbol("THIS", 3);
  put_symbol("THAT", 4);
  put_symbol("SCREEN", 16384);
  put_symbol("KBD", 24576);
  
  return;
}

void free_sym_table() {
  // 23 is somewhat of a 'magic number'. It is the number of pre-defined variables.
  // Because of our strategy in intitializing the sym-table, string literals will be
  // unable to be freed until end of program life so no need to call free here. 
  for (int i = 23; i < SYM_TABLE->size; i++) {
    free(SYM_TABLE->data[i].key);
  }
  free (SYM_TABLE->data);
  free(SYM_TABLE);
  return;
}

void free_parsed_tokens(TOKEN_ARRAY* token_array) {
  for (int i = 0; i < token_array->size; i++) {
    TOKEN entry = token_array->data[i];
    if (strcmp(entry.type, C_TYPE_ASSIGN) == 0 
    || strcmp(entry.type, C_TYPE_JUMP) == 0) {
      free(entry.dest);
      free(entry.comp);
      free(entry.jump);
    }
  }

  free(token_array->data);
  free(token_array);
}

void init_parsed_tokens(void) {
  PARSED_TOKENS = malloc(sizeof(TOKEN_ARRAY));
  TOKEN* tokens = malloc(sizeof(TOKEN) * MAX_TOKEN_SIZE);

  if (PARSED_TOKENS == NULL) {
    printf("Unable to allocate space for parsed tokens\n");
    free(PARSED_TOKENS);
    free(tokens);
    exit(1);
  } 

  PARSED_TOKENS->size = 0;
  PARSED_TOKENS->data = tokens; 
}

int get_symbol(char* key) 
{
  for (int i = 0; i < SYM_TABLE->size; i++)
  {
    SYMBOL curr = SYM_TABLE->data[i];
    if (strcmp(curr.key, key) == 0)
    {
      return curr.value;
    }
  }
  return -1;
};


void put_symbol(char* key, int value) {
  printf("Putting symbol\n");
  printf("Max table size %i\n", MAX_TABLE_SIZE);
  printf("current table size %i\n", SYM_TABLE->size);
  if (SYM_TABLE->size == MAX_TABLE_SIZE) {
    MAX_TABLE_SIZE *= 2;
    SYMBOL* tmp = realloc(SYM_TABLE->data, sizeof(SYMBOL) * MAX_TABLE_SIZE);
    if (tmp == NULL) {
      printf("unable to expand symtable at line %i\n", LINE_COUNT);
      free(SYM_TABLE);
      exit(1);
    }

    SYM_TABLE->data = tmp;
  }

  SYMBOL new_entry;
  new_entry.key = key;
  new_entry.value = value;

  SYM_TABLE->data[SYM_TABLE->size] = new_entry;
  SYM_TABLE->size++;
}

void put_token(TOKEN new_entry) {
  if (PARSED_TOKENS->size == MAX_TOKEN_SIZE) {
    MAX_TOKEN_SIZE *= 2;
    TOKEN* tmp = realloc(PARSED_TOKENS->data, sizeof(TOKEN) * MAX_TOKEN_SIZE);
    if (tmp == NULL) {
      printf("Unable to allcocate for new Token\n");
      free(tmp);
      free(SYM_TABLE->data); 
      free(SYM_TABLE);
      free(PARSED_TOKENS->data);
      free(PARSED_TOKENS);
      exit(1);
    }
    PARSED_TOKENS->data = tmp;
  }

  PARSED_TOKENS->data[PARSED_TOKENS->size] = new_entry;
  PARSED_TOKENS->size++;
}
/////////////////////////// INITALIZERS /////////////////////////////////////



////////////////////// Old attempts :( //////////////////////////////////////
/// 

// char* read_line(FILE* source) {
//   // characters to malloc at first;
//   int read_max = 50; 
//   int read_length = 0;
//   char* text = malloc(sizeof(char) * read_max);
//   char char_buffer = '\0';

//   while(fread(&char_buffer, sizeof(char), 1, source) && char_buffer != EOL && char_buffer != RETURN) {

//     if (char_buffer == SPACE) {
//       skip_space(&char_buffer, source);
//     }

//     if (char_buffer == COMMENT) {
//       skip_comment(&char_buffer, source);
//     }

//     if (char_buffer == EOL || char_buffer == RETURN) {
//       break; 
//     }

//     // check if we need to resize,
//     if (read_length == read_max) {
//       read_max *= 2;
//       char* tmp = realloc(text, sizeof(char) * read_max);
//       if (tmp == NULL) {
//         printf("Unable to alloc read line from line numebr %i", LINE_COUNT);
//         free(text);
//         exit(1); 
//       }
//       text = tmp;
//     }

//     text[read_length] = char_buffer;
//     read_length++;
//   }
//   // add null terminator and end of line char.
//   read_line[read_length] = '\0';

//   return read_line; 
// }