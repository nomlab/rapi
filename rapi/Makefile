CC = mpicc
CFLAGS = -O3 -Wall -Wextra

.PHONY: clean

rapi.so: rapi.c
	$(CC) $(CFLAGS) -fPIC -shared -o $@ $^

clean:
	rm -f *.o *.so
