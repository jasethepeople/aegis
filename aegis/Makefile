CC = clang
CFLAGS = -fsanitize=address,undefined -fsanitize-coverage=trace-pc-guard -g -O1
LDFLAGS = -fsanitize=address,undefined

all: harness
harness: examples/harness.c
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)
clean:
	rm -f harness
.PHONY: all clean
