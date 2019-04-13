BITS 32

section .text
global _start
_start:

fork:
  mov eax, 2    ; fork
  int 0x80

abort:
  test eax, eax
  jnz message
  mov eax, 20    ; getpid
  int 0x80
  mov ebx, eax
  mov eax, 37    ; kill 
  mov ecx, 6     ; SIGABRT
  int 0x80

message:
  mov edx, len   ; string length
  mov ecx, msg   ; string
  mov ebx, 1     ; stdout
  mov eax, 4     ; write
  int 0x80

crash:
  mov [0xbad], byte 1

exit:
  mov eax, 1
  int 0x80

section .data
msg db 'time to crash', 0x0a
len equ $- msg
