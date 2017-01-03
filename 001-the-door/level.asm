
; Level code for the Hakka level, "The Door"

LCD_MEMORY = $D000
KEYPAD_BUTTON_REGISTER = $D0FF
KEYPAD_ISR = $D100
LCD_ISR = $D101

LCD_PWR = $D800
LCD_CTRL_REGISTER = $D801

BUTTON = $B000

.ORG $C000

SEI     ; Disable interrupts while we setup the IRQ

; IRQ lives at $C800
LDA #$C8
STA $FFFF

CLI

; Clear the LCD memory
LDA #$01
STA LCD_CTRL_REGISTER
LDA #$FF
STA LCD_ISR
BRK

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
; First lets check if we're already at 4 numbers and skip
; this entire section if we are
CPY #$04
BEQ HandleKeypadEnd

LDA KEYPAD_BUTTON_REGISTER
CLC               ; Always clear the carry flag before adding
CLD               ; And the decimal flag just in case
ADC #$30          ; Convert the number to an ASCII char representing the number
STA LCD_MEMORY,Y  ; Push the button press to memory
INY

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
BNE HandleLcdEnd
; Clear the LCD memory
LDY #$0A
LDA #$00 
LCD_CLEAR_LOOP
DEY
STA LCD_MEMORY,Y
BNE LCD_CLEAR_LOOP
LDY #$00
HandleLcdEnd
LDA #$00
STA LCD_CTRL_REGISTER
PLA
JMP IRQ_END

IRQ_END
RTI