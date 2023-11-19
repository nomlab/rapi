INSTALL = install -v
INSTALL_DIR = /usr/local/bin
INSTALL_LOCAL_DIR = $(HOME)/.local/bin

CC = mpicc
CFLAGS = -O3 -Wall -Wextra

.PHONY: all clean install install-local

all: rapi.so

rapi.so: rapi.c
	$(CC) $(CFLAGS) -fPIC -shared -o $@ $^

install: all
	$(INSTALL) rapi.so $(INSTALL_DIR)

install-local: all
	$(INSTALL) rapi.so $(INSTALL_LOCAL_DIR)

clean:
	rm -f *.o *.so

