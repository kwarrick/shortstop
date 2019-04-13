CFLAGS=-Wall -Werror -std=c89 -save-temps -m32 -pedantic-errors -O0

hello: hello.c

crash: crash.s
	nasm -f elf32 crash.s
	ld -melf_i386 -o crash crash.o

simple:
	nasm -f elf64 simple.s
	ld -m elf_x86_64 -o simple simple.o

clean:
	rm -fv hello hello.i hello.s hello.o
	rm -fv crash crash.o
	rm -fv simple simple.o
