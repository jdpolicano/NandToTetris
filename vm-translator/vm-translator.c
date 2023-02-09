#include <stdio.h>
#include <string.h>
#include <time.h>
#include "parser.h"
#include "code-writer.h"

static char* get_file_name(char* file_path);

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

    char* file_name = get_file_name(argv[1]);
    TOKEN_ARRAY* parsed_tokens = parse(instruction_file, file_name);
    translate(parsed_tokens, output_file);

    clock_t end = clock();
    double time_spent = (double)(end - begin) / CLOCKS_PER_SEC;
    printf("Time: %fms\n", time_spent);
    return 0;
} 

static char* get_file_name(char* file_path) {
    int path_length = strlen(file_path) + 1;
    char *file_name = malloc(sizeof(char) * path_length);

    if (file_name == NULL) {
        printf("Error in alloc mem for file name\n");
        exit(1);
    }

    int found_extension = 0;
    int file_name_index = 0;

    for (int i = path_length - 1; i >= 0 && (file_path[i] != '/' && file_path[i] != '\\'); i--) {
        if (found_extension) {
            file_name[file_name_index] = file_path[i];
            file_name_index++;
        }
        // start writing data in next iteration
        if (file_path[i] == '.') {
            found_extension = 1;
        }
    }

    int left = 0;
    int right = file_name_index - 1;

    while(left < right) {
        char temp = file_name[left];
        file_name[left] = file_name[right];
        file_name[right] = temp;
        left++;
        right--;
    }

    // null terminate; 
    file_name[file_name_index] = '\0';

    return file_name;
}

