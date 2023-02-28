#include <stdio.h>
#include <string.h>
#include <dirent.h>
#include <errno.h>
#include <sys/stat.h>
#include "parser.h"
#include "code-writer.h"

static char* get_file_name(char* file_path);
static void process_file(char* file_path, FILE* output_file);
static void process_directory(char* directory_path, FILE* output_file);

int main(int argc, char* argv[]) {

    if (argc < 3) {
        printf("Usage: ./assembler [target] [output]\n");
        return 1;
    }

    FILE* output_file = fopen(argv[2], "w");
    if (output_file == NULL)
    {
        printf("unable to open output file name :: %s\n", argv[2]);
        return 2;
    }

    struct stat statbuff;
    if (stat(argv[1], &statbuff) < 0) {
        printf("unable to open file name :: %s\n", argv[1]);
        return 3;
    }

    if ((statbuff.st_mode & S_IFMT) == S_IFREG) {
        // Attempt to read file as single file
        process_file(argv[1], output_file);
    } else if ((statbuff.st_mode & S_IFMT) == S_IFDIR) {
        process_directory(argv[1], output_file);
    } else {
        printf("path is not a file or directory\n");
        return 4;
    }

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

static void process_file(char* file_path, FILE* output_file) {
    printf("Processing file: %s\n", file_path);
    FILE* instruction_file = fopen(file_path, "r");
    if (instruction_file == NULL) {
        printf("Error opening file: %s\n", strerror(errno));
        exit(5);
    }

    char* file_name = get_file_name(file_path);
    TOKEN_ARRAY* parsed_tokens = parse(instruction_file, file_name);
    translate(parsed_tokens, output_file);
    free(file_name);
    free_token_array(parsed_tokens);
    return;
}

static void process_directory(char* directory_path, FILE* output_file) {
    DIR* dir = opendir(directory_path);
    if (dir == NULL) {
        printf("Error opening directory: %s\n", strerror(errno));
        exit(3);
    }

    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_REG || entry->d_type == DT_UNKNOWN) {
            char* file_name = entry->d_name;
            int file_name_length = strlen(file_name);
            int directory_path_length = strlen(directory_path);
            int extension_index = file_name_length > 3 ? file_name_length - 3 : 0;
            
            // Ensure current file has appropriate extension. 
            if (strcmp(&file_name[extension_index], ".vm") != 0) {
                continue;
            }

            char* file_path = malloc(sizeof(char) * (directory_path_length + file_name_length + 2));
            if (file_path == NULL) {
                printf("Error in alloc mem for file path\n");
                exit(4);
            }
            // Create full file path to read.
            sprintf(file_path, "%s/%s", directory_path, file_name);
            process_file(file_path, output_file);
            free(file_path);
        }
    }
}