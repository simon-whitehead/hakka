
; Level code for the Hakka level, "The Door"

HARDWARE_CODE = $C000
HARDWARE_MEMORY = $D000

LCD_MEMORY = $D000

HARDWARE_REG_BUTTON = $D0FF

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

LDA HARDWARE_REG_BUTTON
CMP #$00    ; Is there a button press happening?
BCS HandleButtonPress   ; Yep, a button was pressed
JMP IRQ_END

HandleButtonPress
ADC #$30
STA LCD_MEMORY,Y  ; Push the button press to memory
INY

LDA #$00  
STA HARDWARE_REG_BUTTON ; Clear hardware register

IRQ_END
RTI