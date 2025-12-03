.PHONY: all clean test

CFLAGS = $(shell pkg-config --cflags glib-2.0 gobject-2.0)
LIBS = $(shell pkg-config --libs glib-2.0 gobject-2.0)

all: test

test: calculator.h
	$(CC) $(CFLAGS) test.c -o test -Ltarget/debug -lc_integration_test $(LIBS) -Wl,-rpath,$(CURDIR)/target/debug
	./test

calculator.h:
	cargo build

clean:
	rm -f test calculator.h
	cargo clean
