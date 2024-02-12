; Draw a basic heart to the screen
; A direction of 0 means up and 1 means down

define ly v0
define ry v1
define slen 14
define ldir v2
define rdir v3
define waitnum vc
define tmp vd
define one ve

ld ly 6
ld ry 6
ld ldir 0
ld rdir 1
ld waitnum 6
ld one 1

mainloop:
cls
call drawleft
se ldir 0
jmp moveleftdown

moveleftup:
sub ly one
jmp skipmoveleftdown
moveleftdown:
add ly one
skipmoveleftdown:

call drawright
se rdir 0
jmp moverightdown

moverightup:
sub ry one
jmp skipmoverightdown
moverightdown:
add ry one
skipmoverightdown:

; If ldir == 1 && ly + slen == 32, then set ldir = 0
se ldir 1
jmp skipleftatbottom
ld tmp ly
add tmp slen
sne tmp 32
ld ldir 0
skipleftatbottom:

; If ldir == 0 && ly == 0, then set ldir = 1
se ldir 0
jmp skipleftattop
sne ly 0
ld ldir 1
skipleftattop:

; If rdir == 1 && ry + slen == 32, then set rdir = 0
se rdir 1
jmp skiprightatbottom
ld tmp ry
add tmp slen
sne tmp 32
ld rdir 0
skiprightatbottom:

; If rdir == 0 && ry == 0, then set rdir = 1
se rdir 0
jmp skiprightattop
sne ry 0
ld rdir 1
skiprightattop:

; Loop forever
call setwaitnum
delay waitnum
wait:
ld vf dt
se vf 0
jmp wait
jmp mainloop

; Draw the left half of the heart.
; Modifies: I, tmp
; Needs set: ly
drawleft:
	ld I, lefthalf
	ld tmp, 23
	draw tmp, ly, slen
	ret

; Draw the right half of the heart.
; Modifies: I, tmp
; Needs set: ry
drawright:
	ld I, righthalf
	ld tmp, 31
	draw tmp, ry, slen
	ret

; Test for every key being pressed and set the waitnum register if a new key is pressed
; Modifies: waitnum, tmp
setwaitnum:
	ld tmp 0
	sknp tmp
	ld waitnum 0

	ld tmp 1
	sknp tmp
	ld waitnum 1

	ld tmp 2
	sknp tmp
	ld waitnum 2

	ld tmp 3
	sknp tmp
	ld waitnum 3

	ld tmp 4
	sknp tmp
	ld waitnum 4

	ld tmp 5
	sknp tmp
	ld waitnum 5

	ld tmp 6
	sknp tmp
	ld waitnum 6

	ld tmp 7
	sknp tmp
	ld waitnum 7

	ld tmp 8
	sknp tmp
	ld waitnum 8

	ld tmp 9
	sknp tmp
	ld waitnum 9

	ld tmp 10
	sknp tmp
	ld waitnum 10

	ld tmp 11
	sknp tmp
	ld waitnum 11

	ld tmp 12
	sknp tmp
	ld waitnum 12

	ld tmp 13
	sknp tmp
	ld waitnum 13

	ld tmp 14
	sknp tmp
	ld waitnum 14

	ld tmp 15
	sknp tmp
	ld waitnum 15

	ret

lefthalf:
db  %00111100,
    %01000010,
    %10000001,
    %10000000,
    %10000000,
    %10000000,
    %10000000,
    %01000000,
    %00100000,
    %00010000,
    %00001000,
    %00000100,
    %00000010,
    %00000001

righthalf:
db  %00111100,
    %01000010,
    %10000001,
    %00000001,
    %00000001,
    %00000001,
    %00000001,
    %00000010,
    %00000100,
    %00001000,
    %00010000,
    %00100000,
    %01000000,
    %10000000,
