#include <stdio.h>
#include <fcntl.h>

int main() {
    printf("O_RDONLY = %08x\n", O_RDONLY);
    printf("F_GETFL = %08x\n", F_GETFL);
    printf("F_SETFL = %08x\n", F_SETFL);
    printf("O_APPEND = %08x\n", O_APPEND);
    printf("O_NONBLOCK = %08x\n", O_NONBLOCK);
    printf("O_ASYNC = %08x\n", O_ASYNC);
}
