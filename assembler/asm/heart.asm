; Draw a basic heart to the screen

cls

; Draw the left half of the heart
ld I, lefthalf
ld v0, 23
ld v1, 7
draw v0, v1, 14

; Draw the right half of the heart
ld I, righthalf
ld v0, 31
ld v1, 7
draw v0, v1, 14

; Loop forever
spin:
jmp spin

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
