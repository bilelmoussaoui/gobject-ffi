.PHONY: all clean run build

CFLAGS = $(shell pkg-config --cflags glib-2.0 gobject-2.0)
LIBS = $(shell pkg-config --libs glib-2.0 gobject-2.0)

all: run

build:
	cargo build

test: build test.c
	$(CC) $(CFLAGS) test.c -o test -Ltarget/debug -lc_integration_test $(LIBS) -Wl,-rpath,$(CURDIR)/target/debug

run: test
	./test

clean:
	rm -f test calculator.h
	cargo clean
