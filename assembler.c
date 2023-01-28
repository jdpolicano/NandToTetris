#include <stdio.h>
#include <string.h>
#include <time.h>
#include "parser.h"

int main(int argc, char* argv[])
{
  clock_t begin = clock();

  /* here, do your time-consuming job */
  FILE* instruction_file = fopen("../add/Add.asm", "r");
  if (instruction_file == NULL)
  {
    printf("No file found at ../Add.asm, sorry.\n");
    return 1;
  }
  printf("Parsing file\n");
  TOKEN_ARRAY* result = parse(instruction_file);

  // for (int i = 0; i < result->size; i++) {
  //   TOKEN entry = result->data[i];

  //   if (strcmp(entry.type, A_TYPE) == 0) {
  //     printf("type -> %s\n", entry.type);
  //     printf("value -> %i\n", entry.data);
  //   } else {
  //     printf("type -> %s\n", entry.type);
  //     printf("dest -> %s\n", entry.dest);
  //     printf("comp -> %s\n", entry.comp);
  //     printf("jump -> %s\n", entry.jump);
  //   }
  // }

  printf("Size -> %i\n", result->size);
  free_parsed_tokens(result);
  clock_t end = clock();
  double time_spent = (double)(end - begin) / CLOCKS_PER_SEC;

  printf("Time: %fms\n", time_spent);
  return 0;
}