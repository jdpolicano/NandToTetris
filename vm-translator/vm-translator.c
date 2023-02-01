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

// to-do - add more robust schema enforcement for file name, capitalization, etc...
  FILE* instruction_file = fopen(argv[1], "r");
  if (instruction_file == NULL)
  {
    printf("No file found at %s, sorry.\n", argv[1]);
    return 1;
  }

  FILE* output_file = fopen(argv[2], "w");
  if (output_file == NULL)
  {
    printf("No file found at %s, sorry.\n", argv[2]);
    return 1;
  }


  clock_t end = clock();
  double time_spent = (double)(end - begin) / CLOCKS_PER_SEC;
  printf("Time: %fms\n", time_spent);

  return 0;
} 