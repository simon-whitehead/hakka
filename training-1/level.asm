
; This code runs Level 1 of the game, HAKKA.

; 16-bit Ship X position
X_0 = $00
X_1 = $01

; 16-bit Ship Y position
Y_0 = $02
Y_1 = $03

; The keycode of the last key pressed
KEY = $04

; 16-bit movement speed
MOV_0 = $05
MOV_1 = $06

GameLoop

SEI
JSR RightArrow
JSR LeftArrow
JSR AdjustOffscreen
CLI

JMP GameLoop

RightArrow:

LDA KEY
CMP #39 ; Was the right arrow pressed?
BNE RightArrowEnd

CLC
LDA X_0
ADC MOV_0
STA X_0
LDA X_1
ADC MOV_1
STA X_1

RightArrowEnd:
RTS

LeftArrow:

LDA KEY
CMP #37 ; Was the left arrow pressed?
BNE LeftArrowEnd

SEC
LDA X_0
SBC MOV_0
STA X_0
LDA X_1
SBC MOV_1
STA X_1

LeftArrowEnd:
RTS

AdjustOffscreen:
LDA X_1
CMP #$F0
BCC EndAdjust
LDA #$00
STA X_0
STA X_1

EndAdjust:
RTS

END: