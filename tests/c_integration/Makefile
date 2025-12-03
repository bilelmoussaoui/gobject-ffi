.PHONY: all clean run build

CFLAGS = $(shell pkg-config --cflags glib-2.0 gobject-2.0)
LIBS = $(shell pkg-config --libs glib-2.0 gobject-2.0)

TARGET_DIR = $(shell test -f ../../target/debug/libc_integration_test.so && echo "../../target/debug" || echo "target/debug")

all: run

build:
	cargo build

test: build test.c
	$(CC) $(CFLAGS) test.c -o test -L$(TARGET_DIR) -lc_integration_test $(LIBS) -Wl,-rpath,$(abspath $(TARGET_DIR))

run: test
	./test

clean:
	rm -f test calculator.h
	cargo clean
