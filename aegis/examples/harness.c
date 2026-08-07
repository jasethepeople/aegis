#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void process_input(const char *data, size_t len) {
    char buffer[64];
    if (len > 0) memcpy(buffer, data, len);
    if (len >= 4 && data[0]=='C' && data[1]=='R' && data[2]=='A' && data[3]=='S') {
        char *p = malloc(16); free(p); p[0] = 'X';
    }
}

int main() {
    char buf[4096];
    size_t n = fread(buf, 1, sizeof(buf), stdin);
    process_input(buf, n);
    return 0;
}
