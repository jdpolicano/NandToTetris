#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "parser.h"

#define REGISTER_SIZE 16
#define C_TYPE_HEADER_START 0
#define C_TYPE_HEADER_END 3
#define C_TYPE_COMP_START 3
#define C_TYPE_COMP_END 10
#define C_TYPE_DEST_START 10
#define C_TYPE_DEST_END 13
#define C_TYPE_JUMP_START 13
#define C_TYPE_JUMP_END 16
#define C_TYPE_DEST_SIZE 3
#define C_TYPE_JUMO_SIZE 3


// this will be returned from our assemble function to indicate how many lines of 
// binary were written; Should be the same as the size of token array. 
int LINE_COUNT = 0;


int assemble(FILE* target, TOKEN_ARRAY* tokens);
char* build_a_instruction(char* empty_register, int address);
char* build_c_instruction(char* empty_register, TOKEN c_type);

char* route_c_type_comp(char* empty_register, TOKEN token);
char* route_c_type_dest(char* empty_register, TOKEN token);
char* route_c_type_jump(char* empty_register, TOKEN token);

char* build_c_type_comp(char* empty_register, char* write);
char* build_c_type_dest(char* empty_register, char* write);
char* build_c_type_jump(char* empty_register, char* write);

char* build_c_type_header(char* empty_register);
char* build_empty_register(void);

// transform - takes in a tagret file and a token array and writes hack instructions;
// returns an int, 0 if failed, and other number if success. 
int assemble(FILE* target, TOKEN_ARRAY* tokens) {
    printf("Writing binaries\n");
    for (int i = 0; i < tokens->size; i++) {
        char* empty_reg = build_empty_register();
        TOKEN token = tokens->data[i];
        if (strcmp(token.type, A_TYPE) == 0) {
            build_a_instruction(empty_reg, token.data);
            fwrite(empty_reg, sizeof(char), REGISTER_SIZE + 1, target);
            free(empty_reg);
            continue;
        } else {
            build_c_instruction(empty_reg, token);
            fwrite(empty_reg, sizeof(char), REGISTER_SIZE + 1, target);
            free(empty_reg);
            continue;
        }
    }
    printf("Wrote binaries to disk\n");
    return 1;
}

// build a binary representation of an a instruciton. 16 bit values used for this creation;
char* build_a_instruction(char* empty_register, int address) {
    for(int i = 0; i < REGISTER_SIZE; i++) {
        empty_register[REGISTER_SIZE - i - 1] = (address % 2 == 0) ? '0' : '1';
        address = address >> 1;
    }

    return empty_register; 
}


char* build_c_instruction(char* empty_register, TOKEN c_type) {
    build_c_type_header(empty_register);
    if (strcmp(c_type.type, C_TYPE_ASSIGN) == 0) {
        route_c_type_dest(empty_register, c_type);
        route_c_type_comp(empty_register, c_type);
        return empty_register;
    } else {
        route_c_type_comp(empty_register, c_type);
        route_c_type_jump(empty_register, c_type);
        return empty_register;
    } 
}

char* build_c_type_header(char* empty_register) {
    // headers for atypes start conventionally with three 111s. 
    for (int i = C_TYPE_HEADER_START; i < C_TYPE_HEADER_END; i++) {
        empty_register[i] = '1';
    }

    return empty_register;
}

// This function feels absolutely ridiculous...
char* route_c_type_comp(char* empty_register, TOKEN token) {
    if (strcmp(token.comp, "0") == 0) return build_c_type_comp(empty_register, "0101010");
    if (strcmp(token.comp, "1") == 0) return build_c_type_comp(empty_register, "0111111");
    if (strcmp(token.comp, "-1") == 0) return build_c_type_comp(empty_register, "0111010");
    if (strcmp(token.comp, "D") == 0) return build_c_type_comp(empty_register, "0001100");
    if (strcmp(token.comp, "A") == 0) return build_c_type_comp(empty_register, "0110000");
    if (strcmp(token.comp, "M") == 0) return build_c_type_comp(empty_register, "1110000");
    if (strcmp(token.comp, "!D") == 0) return build_c_type_comp(empty_register, "0001101");
    if (strcmp(token.comp, "!A") == 0) return build_c_type_comp(empty_register, "0110001");
    if (strcmp(token.comp, "!M") == 0) return build_c_type_comp(empty_register, "1110001");
    if (strcmp(token.comp, "-D") == 0) return build_c_type_comp(empty_register, "0001111");
    if (strcmp(token.comp, "-A") == 0) return build_c_type_comp(empty_register, "0110011");
    if (strcmp(token.comp, "-M") == 0) return build_c_type_comp(empty_register, "1110011");
    if (strcmp(token.comp, "D+1") == 0) return build_c_type_comp(empty_register, "0011111");
    if (strcmp(token.comp, "A+1") == 0) return build_c_type_comp(empty_register, "0110111");
    if (strcmp(token.comp, "M+1") == 0) return build_c_type_comp(empty_register, "1110111");
    if (strcmp(token.comp, "D-1") == 0) return build_c_type_comp(empty_register, "0001110");
    if (strcmp(token.comp, "A-1") == 0) return build_c_type_comp(empty_register, "0110010");
    if (strcmp(token.comp, "M-1") == 0) return build_c_type_comp(empty_register, "1110010");
    if (strcmp(token.comp, "D+A") == 0) return build_c_type_comp(empty_register, "0000010");
    if (strcmp(token.comp, "D+M") == 0) return build_c_type_comp(empty_register, "1000010");
    if (strcmp(token.comp, "D-A") == 0) return build_c_type_comp(empty_register, "0010011");
    if (strcmp(token.comp, "D-M") == 0) return build_c_type_comp(empty_register, "1010011");
    if (strcmp(token.comp, "A-D") == 0) return build_c_type_comp(empty_register, "0000111");
    if (strcmp(token.comp, "M-D") == 0) return build_c_type_comp(empty_register, "1000111");
    if (strcmp(token.comp, "D&A") == 0) return build_c_type_comp(empty_register, "0000000");
    if (strcmp(token.comp, "D&M") == 0) return build_c_type_comp(empty_register, "1000000");
    if (strcmp(token.comp, "D|A") == 0) return build_c_type_comp(empty_register, "0010101");
    if (strcmp(token.comp, "D|M") == 0) return build_c_type_comp(empty_register, "1010101");
    
    printf("Something is wrong, oops\n");
    return empty_register; 
}

// strategy for destination is slighly different because syntax is flexible towards ordering of assignments...
char* route_c_type_dest(char* empty_register, TOKEN token) {
    int dest_length = strlen(token.dest);
    char* empty_dest = malloc(C_TYPE_DEST_SIZE);
    memset(empty_dest, '0', C_TYPE_DEST_SIZE);

    for (int i = 0; i < dest_length; i++) {
        if (token.dest[i] == 'M') {
            empty_dest[2] = '1';
        }

        else if (token.dest[i] == 'D') {
            empty_dest[1] = '1';
        }

        else if (token.dest[i] == 'A') {
            empty_dest[0] = '1';
        }
    }

    build_c_type_dest(empty_register, empty_dest);
    free(empty_dest);
    return empty_register;
}

char* route_c_type_jump(char* empty_register, TOKEN token) {
    if(strcmp(token.jump, "null") == 0) return build_c_type_jump(empty_register, "000");
    if(strcmp(token.jump, "JGT") == 0) return build_c_type_jump(empty_register, "001");
    if(strcmp(token.jump, "JEQ") == 0) return build_c_type_jump(empty_register, "010");
    if(strcmp(token.jump, "JGE") == 0) return build_c_type_jump(empty_register, "011");
    if(strcmp(token.jump, "JLT") == 0) return build_c_type_jump(empty_register, "100");
    if(strcmp(token.jump, "JNE") == 0) return build_c_type_jump(empty_register, "101");
    if(strcmp(token.jump, "JLE") == 0) return build_c_type_jump(empty_register, "110");
    if(strcmp(token.jump, "JMP") == 0) return build_c_type_jump(empty_register, "111");
    return empty_register;
}

char* build_c_type_comp(char* empty_register, char* write) {
    for (int i = C_TYPE_COMP_START; i < C_TYPE_COMP_END; i++) {
        empty_register[i] = write[i - C_TYPE_COMP_START]; 
    }
    return empty_register;
}

char* build_c_type_dest(char* empty_register, char* write) {
    for (int i = C_TYPE_DEST_START; i < C_TYPE_DEST_END; i++) {
        empty_register[i] = write[i - C_TYPE_DEST_START]; 
    }
    return empty_register;
}

char* build_c_type_jump(char* empty_register, char* write) {
    for (int i = C_TYPE_JUMP_START; i < C_TYPE_JUMP_END; i++) {
        empty_register[i] = write[i - C_TYPE_JUMP_START]; 
    }
    return empty_register;
}

// build an empty c string to build into binary instruct for final output;
char* build_empty_register(void) {
    char* new_reg = malloc(REGISTER_SIZE + 2);
    if (new_reg == NULL) {
        printf("Unable to alloc mem for new register\n");
        free(new_reg);
        exit(1);
    }

    memset(new_reg, '0', REGISTER_SIZE);

    new_reg[REGISTER_SIZE] = '\n';
    new_reg[REGISTER_SIZE + 1] = '\0';

    return new_reg;
}