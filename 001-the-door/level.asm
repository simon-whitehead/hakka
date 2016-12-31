
; Level code for the Hakka level, "The Door"

HARDWARE_CODE = $C000
HARDWARE_MEMORY = $D000

HARDWARE_REG_BUTTON = $D0FF

BUTTON = $B000

.ORG $C000

SEI     ; Disable interrupts while we setup the IRQ

LDA #$C8
STA $FFFF

CLI

GameLoop

JMP GameLoop

.ORG $C800

LDA HARDWARE_REG_BUTTON
CMP #$00    ; Is there a button press happening?
BCS HandleButtonPress   ; Yep, a button was pressed
JMP IRQ_END

HandleButtonPress
STA BUTTON  ; Push the button press to memory
EOR BUTTON  
STA HARDWARE_REG_BUTTON ; Clear hardware register

IRQ_END
RTI