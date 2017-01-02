
; Level code for the Hakka level, "The Door"

LCD_MEMORY = $D000
KEYPAD_BUTTON_REGISTER = $D0FF
KEYPAD_ISR = $D100

BUTTON = $B000

.ORG $C000

SEI     ; Disable interrupts while we setup the IRQ

; IRQ lives at $C800
LDA #$C8
STA $FFFF

CLI

; Clear the LCD memory
LDY #$09
LDA #$00 
LCD_CLEAR_LOOP
STA LCD_MEMORY,Y
DEY
BNE LCD_CLEAR_LOOP
LDY #$00

MAIN

JMP MAIN

.ORG $C800

; Test what interrupt happened
BIT KEYPAD_ISR
BMI HandleKeypad
JMP IRQ_END

HandleKeypad
; First lets check if we're already at 4 numbers and skip
; this entire section if we are
CPY #$04
BEQ IRQ_END

LDA KEYPAD_BUTTON_REGISTER
CMP #$00    ; Is there a button press happening?
BCS HandleButtonPress   ; Yep, a button was pressed
JMP IRQ_END

HandleButtonPress
CLC               ; Always clear the carry flag before adding
CLD               ; And the decimal flag just in case
ADC #$30          ; Convert the number to an ASCII char representing the number
STA LCD_MEMORY,Y  ; Push the button press to memory
INY

LDA #$00  
STA KEYPAD_BUTTON_REGISTER ; Clear hardware register

IRQ_END
RTI