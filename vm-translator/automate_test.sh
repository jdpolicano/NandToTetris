echo "\nBegining script"
EXEC=./translator
ROOT_DIR=../examples/Virtual

# Subdirectories
MEMORY_ACCESS=MemoryAccess
STACK_ARITHMETIC=StackArithmetic
PROGRAM_FLOW=ProgramFlow
FUNCTION_CALLS=FunctionCalls

# program directories
BASIC=BasicTest
POINTER=PointerTest
STATIC=StaticTest
SIMPLE_ADD=SimpleAdd
STACK_TEST=StackTest
BASIC_LOOP=BasicLoop
FIBONACCI_SERIES=FibonacciSeries
FIBONACCI_ELEMENT=FibonacciElement
NESTED_CALL=NestedCall
SIMPLE_FUNCTION=SimpleFunction
STATICS_TEST=StaticsTest

BASIC_INPUT=$ROOT_DIR/$MEMORY_ACCESS/$BASIC/$BASIC.vm
BASIC_OUTPUT=$ROOT_DIR/$MEMORY_ACCESS/$BASIC/$BASIC.asm
POINTER_INPUT=$ROOT_DIR/$MEMORY_ACCESS/$POINTER/$POINTER.vm
POINTER_OUTPUT=$ROOT_DIR/$MEMORY_ACCESS/$POINTER/$POINTER.asm
STATIC_INPUT=$ROOT_DIR/$MEMORY_ACCESS/$STATIC/$STATIC.vm
STATIC_OUTPUT=$ROOT_DIR/$MEMORY_ACCESS/$STATIC/$STATIC.asm
SIMPLE_ADD_INPUT=$ROOT_DIR/$STACK_ARITHMETIC/$SIMPLE_ADD/$SIMPLE_ADD.vm
SIMPLE_ADD_OUTPUT=$ROOT_DIR/$STACK_ARITHMETIC/$SIMPLE_ADD/$SIMPLE_ADD.asm
STACK_TEST_INPUT=$ROOT_DIR/$STACK_ARITHMETIC/$STACK_TEST/$STACK_TEST.vm
STACK_TEST_OUTPUT=$ROOT_DIR/$STACK_ARITHMETIC/$STACK_TEST/$STACK_TEST.asm
BASIC_LOOP_INPUT=$ROOT_DIR/$PROGRAM_FLOW/$BASIC_LOOP/$BASIC_LOOP.vm
BASIC_LOOP_OUTPUT=$ROOT_DIR/$PROGRAM_FLOW/$BASIC_LOOP/$BASIC_LOOP.asm
FIBONACCI_SERIES_INPUT=$ROOT_DIR/$PROGRAM_FLOW/$FIBONACCI_SERIES/$FIBONACCI_SERIES.vm
FIBONACCI_SERIES_OUTPUT=$ROOT_DIR/$PROGRAM_FLOW/$FIBONACCI_SERIES/$FIBONACCI_SERIES.asm
FIBBONACCI_ELEMENT_INPUT=$ROOT_DIR/$PROGRAM_FLOW/$FIBONACCI_ELEMENT/$FIBONACCI_ELEMENT.vm
FIBONACCI_ELEMENT_OUTPUT=$ROOT_DIR/$PROGRAM_FLOW/$FIBONACCI_ELEMENT/$FIBONACCI_ELEMENT.asm
NESTED_CALL_INPUT=$ROOT_DIR/$FUNCTION_CALLS/$NESTED_CALL/$NESTED_CALL.vm
NESTED_CALL_OUTPUT=$ROOT_DIR/$FUNCTION_CALLS/$NESTED_CALL/$NESTED_CALL.asm
SIMPLE_FUNCTION_INPUT=$ROOT_DIR/$FUNCTION_CALLS/$SIMPLE_FUNCTION/$SIMPLE_FUNCTION.vm
SIMPLE_FUNCTION_OUTPUT=$ROOT_DIR/$FUNCTION_CALLS/$SIMPLE_FUNCTION/$SIMPLE_FUNCTION.asm
STATICS_TEST_INPUT=$ROOT_DIR/$FUNCTION_CALLS/$STATICS_TEST/$STATICS_TEST.vm
STATICS_TEST_OUTPUT=$ROOT_DIR/$FUNCTION_CALLS/$STATICS_TEST/$STATICS_TEST.asm

echo "\nCompiling executable\n"
make

echo "\nTesting Basic\n"
$EXEC $BASIC_INPUT $BASIC_OUTPUT
echo "\nTesting Pointer\n"
$EXEC $POINTER_INPUT $POINTER_OUTPUT
echo "\nTesting INPUT\n"
$EXEC $STATIC_INPUT $STATIC_OUTPUT
echo "\nTesting Simple Add\n"
$EXEC $SIMPLE_ADD_INPUT $SIMPLE_ADD_OUTPUT
echo "\nTesting StackTest\n"
$EXEC $STACK_TEST_INPUT $STACK_TEST_OUTPUT
echo "\nTesting Basic Loop\n"
$EXEC $BASIC_LOOP_INPUT $BASIC_LOOP_OUTPUT
echo "\nTesting Fibonacci Series\n"
$EXEC $FIBONACCI_SERIES_INPUT $FIBONACCI_SERIES_OUTPUT
echo "\nTesting Fibonacci Element\n"
$EXEC $FIBBONACCI_ELEMENT_INPUT $FIBONACCI_ELEMENT_OUTPUT
echo "\nTesting Nested Call\n"
$EXEC $NESTED_CALL_INPUT $NESTED_CALL_OUTPUT
echo "\nTesting Simple Function\n"
$EXEC $SIMPLE_FUNCTION_INPUT $SIMPLE_FUNCTION_OUTPUT
echo "\nTesting Statics Test\n"
$EXEC $STATICS_TEST_INPUT $STATICS_TEST_OUTPUT

echo "Script end"