#include <stdio.h>
#include <stdlib.h>

void vulnerable_allocation(int size)
{
    char *buffer = (char *)malloc(size);
    // not check if the allocation succeeded
    strcpy(buffer, "Hello, World!");
    // potential overflow
    printf("%s\n", buffer);
    free(buffer);
}

int main()
{
    vulnerable_allocation(12);
    return 0;
}
