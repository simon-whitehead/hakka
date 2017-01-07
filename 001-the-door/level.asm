
; Level code for the Hakka level, "The Door"

LCD_MEMORY = $D000
LCD_DISPLAY_BUFFER_POINTER = $D006
LCD_DISPLAY_INPUT_BUFFER = $D010
LCD_BUFFER_SIZE = $D00F
KEYPAD_ISR = $D100
LCD_ISR = $D101

LCD_PWR = $D800
LCD_CTRL_REGISTER = $D801
KEYPAD_PWR = $D900
KEYPAD_BUTTON_REGISTER = $D901

LCD_PROPERTIES = $E000
ACCESS_DENIED = $E010

.ORG $C000

JSR ClearLcd
JMP INIT

; Clear the LCD memory
ClearLcd
PHA
TYA
PHA
LDY #$10
LDA #$00 
LCD_CLEAR_LOOP
DEY
STA LCD_DISPLAY_INPUT_BUFFER,Y
BNE LCD_CLEAR_LOOP
LDY #$00
PLA
TAY
PLA
RTS

INIT

; Enable the LCD
LDA #$FF
STA LCD_PWR

; Copy the LCD properties into its memory map
LDA #$02
STA LCD_CTRL_REGISTER
LDY #$00
InitLcdLoop
CPY #$06
BEQ InitLcdEnd
LDA LCD_PROPERTIES,Y
STA LCD_MEMORY,Y
INY
JMP InitLcdLoop
InitLcdEnd
LDA #$00
STA LCD_CTRL_REGISTER

; Enable the keypad
LDA #$FF
STA KEYPAD_PWR

MAIN

JMP MAIN

.ORG $C800

; Test what interrupt happened
BIT KEYPAD_ISR
BMI HandleKeypad
BIT LCD_ISR
BMI HandleLcd
JMP IRQ_END

HandleKeypad
PHA
LDA KEYPAD_BUTTON_REGISTER
; Did we press the hash? Lets verify the passcode
CMP #$FF
BEQ HandleSubmission
; Check if we're already at 4 numbers and skip
; this entire section if we are
LDY LCD_BUFFER_SIZE
CPY #$04
BEQ HandleKeypadEnd

CLC               ; Always clear the carry flag before adding
CLD               ; And the decimal flag just in case
ADC #$30          ; Convert the number to an ASCII char representing the number
STA LCD_DISPLAY_INPUT_BUFFER,Y  ; Push the button press to memory
INY
STY LCD_BUFFER_SIZE
JMP HandleKeypadEnd

HandleSubmission
JSR Admin

HandleKeypadEnd
LDA #$00  
STA KEYPAD_BUTTON_REGISTER ; Clear hardware register
STA KEYPAD_ISR             ; Clear the status register
PLA
JMP IRQ_END

HandleLcd
PHA
LDA LCD_CTRL_REGISTER
CMP #$01    ; 0x01 == CLEAR
BEQ ClearLcd
HandleLcdEnd
LDA #$00
STA LCD_CTRL_REGISTER
STA LCD_ISR
PLA
JMP IRQ_END

IRQ_END
RTI

Admin
PHA
TYA
PHA

; CMP
LDY #$00
AdminLoop
LDA $F000,Y
; Check for end
CMP #$00
BEQ AllowAccess
ADC #$30
CMP LCD_DISPLAY_INPUT_BUFFER,Y
BNE DenyAccess
INY
JMP AdminLoop

AdminEnd

DenyAccess
; Set the font color to WHITE and the background to RED
LDA #$02
STA LCD_CTRL_REGISTER
LDA #$FF
STA LCD_MEMORY
LDY #1
STA LCD_MEMORY,Y
LDY #2
STA LCD_MEMORY,Y
LDA #$FF
LDY #3
STA LCD_MEMORY,Y
LDA #$00
LDY #4
STA LCD_MEMORY,Y
LDY #5
STA LCD_MEMORY,Y
LDA #$00
STA LCD_CTRL_REGISTER
; Point the buffer at the ACCESS DENIED message
LDA #$10
STA LCD_DISPLAY_BUFFER_POINTER
LDY #1
LDA #$E0
STA LCD_DISPLAY_BUFFER_POINTER,Y
; Finally, clear the LCD display
JSR ClearLcd

DenyAccessEnd
PLA
TAY
PLA
RTS

AllowAccess
; Set the font color to WHITE and the background to GREEN
LDA #$02
STA LCD_CTRL_REGISTER
LDA #$FF
STA LCD_MEMORY
LDY #1
STA LCD_MEMORY,Y
LDY #2
STA LCD_MEMORY,Y
LDA #$00
LDY #3
STA LCD_MEMORY,Y
LDA #$FF
LDY #4
STA LCD_MEMORY,Y
LDY #5
STA LCD_MEMORY,Y
LDA #$00
STA LCD_CTRL_REGISTER
; Point the buffer at the ACCESS GRANTED message (TODO)
LDA #$20
STA LCD_DISPLAY_BUFFER_POINTER
LDY #1
LDA #$E0
STA LCD_DISPLAY_BUFFER_POINTER,Y
; Finally, clear the LCD display
JSR ClearLcd

AllowAccessEnd
PLA
TAY
PLA
RTS

.ORG $D006
.BYTE #$10, #$D0

.ORG $E000
.BYTE #$C0, #$C0, #$C0, #$00, #$00, #$FF    ; LCD Properties
.ORG $E010
.BYTE #$41, #$43, #$43, #$45, #$53, #$53, #$20, #$44, #$45, #$4E, #$49, #$45, #$44  ; "ACCESS DENIED"
.ORG $E020
.BYTE #$41, #$43, #$43, #$45, #$53, #$53, #$20, #$47, #$52, #$41, #$4E, #$54, #$45, #$44  ; "ACCESS GRANTED"

.ORG $FFFF
.BYTE #$C8