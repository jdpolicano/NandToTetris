#ifndef CODE_H
#define CODE_H

#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "parser.h"

int assemble(FILE* target, TOKEN_ARRAY* tokens);

#endif