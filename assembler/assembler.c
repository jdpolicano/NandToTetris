#include <stdio.h>
#include <string.h>
#include <time.h>
#include "parser.h"
#include "code.h"

int main(int argc, char* argv[])
{
  clock_t begin = clock();

  if (argc < 3) {
    printf("Usage: ./assembler [target] [output]\n");
    return 1;
  }


  /* here, do your time-consuming job */
  FILE* instruction_file = fopen(argv[1], "r");
  if (instruction_file == NULL)
  {
    printf("No file found at ../Add.asm, sorry.\n");
    return 1;
  }

  FILE* output_file = fopen(argv[2], "w");
  if (output_file == NULL)
  {
    printf("No file found at ../Add.asm, sorry.\n");
    return 1;
  }


  printf("Parsing file\n");
  TOKEN_ARRAY* tokens = parse(instruction_file);
  printf("Finished tokenizing source file\n");

  printf("Writing bin to file at %s\n", argv[2]); 
  int write_result = assemble(output_file, tokens);
  printf(write_result ? "Successfully wrote file\n" : "Failed to write file\n");

  clock_t end = clock();
  double time_spent = (double)(end - begin) / CLOCKS_PER_SEC;
  printf("Time: %fms\n", time_spent);
  fclose(instruction_file);
  fclose(output_file);
  free_parsed_tokens(tokens);
  return 0;
}