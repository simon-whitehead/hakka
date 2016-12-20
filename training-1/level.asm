
; This code runs Training Level 1 of the game, hakka.
.ORG $5000

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

JSR UpArrow
JSR DownArrow
JSR Flame

JMP GameLoop

DownArrow:

LDA KEY
CMP #40 ; Was the down arrow pressed?
BNE DownArrowEnd

; Add the movement speed to the Y position
CLC
SEI
LDA Y_0
ADC MOV_0
STA Y_0
LDA Y_1
ADC MOV_1
STA Y_1
CLI

DownArrowEnd:
RTS

UpArrow:

LDA KEY
CMP #38 ; Was the up arrow pressed?
BNE UpArrowEnd

SEC
SEI
LDA Y_0
SBC MOV_0
STA Y_0
LDA Y_1
SBC MOV_1
STA Y_1

; clamp the Y position so it can't be bigger than <figure it out>

UpArrowEnd:
CLI
RTS

; Toggle the flame based on key state
Flame:
LDA KEY
CMP #$00
BEQ FlameOff
LDA #$01
STA $07
JMP FlameEnd
FlameOff:
LDA #$00
STA $07

FlameEnd:
RTS

END: