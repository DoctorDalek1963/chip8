; This program just demonstrates and tests all the instructions in the assembler.
; Don't try to run it. It will probably do strange things.

; Comments start with semicolons
start:				; labels are identifiers followed by colons
	CLS
	JMP	start
	JMP	#123		; Hexadecimal numbers are preceded by # symbols
	JP	v0, #123
	JP	v0, end
	call end
	call #203
	se V1, #AA
	se V2, v3
	sne V1, #AA
	sne V2, v3
end:
	RET
	add V1, #AA
	add V2, v3
	ld V1, #AA
	ld V2, v3
	or V2, v3
	and VA, vb
	xor VA, vb
	shr VA, vb
	shr VA
	subn VA, vb
	shl VA, vb
	shl VA
	rnd VD, #FF
	drw VE, VF, #4
	skp VE
	sknp VA
	add I, V8
	ld I, #AAA
	ld V5, DT
	ld V5, K
	ld DT, V5
	ld ST, V5
	ld F, V5
	ld B, V5
	stor VA
	rstr VA
	;ld [I], VA
	;ld VA, [I]

; "define" can be used to define constants
define aaa #222
	jmp aaa

; "define" can also be used to define aliases for registers
define bbb vd
	ld bbb, %01010101	; Binary literals start with % symbols
	JMP %101001010101
	JMP x
	LD I, x

; Offset moves the location where output is generated
;offset #280

; This is how you can define sprites:
; "db" emits raw bytes, separated by commas.
; "dw" can emit 16-bit words.

x: dw #1122, #3344
y: db
	%00100100,
	%11111111,
	%01011010,
	%00111100,
	%00100100
	CLS

; You can load text data into the output through the `text` directive:
string1:
	text "hello"
; (The label `string1` can be used later like so: `ld I, string1`)

; The above is equivalent to this `db` directive
; db #68, #65, #6C, #6C, #6F, #00

; The string can also contain special symbols escaped with '\'
;string2: text "\"\e\r\n"

; This is how you can include another file:
include "font2.asm"

;@something
