; 15mm square cut test for laser engraver
; Square centered at X20 Y20
; Laser intensity S255, constant power mode
; Single pass at slow speed

G21 ; Units in millimeters
G90 ; Absolute coordinates
G0 X12.5 Y12.5 F500 ; Move to start point (bottom left corner)
M4 S255 ; Laser on at full intensity, constant power
G1 X27.5 Y12.5 F100 ; Bottom side
G1 X27.5 Y27.5 F100 ; Right side
G1 X12.5 Y27.5 F100 ; Top side
G1 X12.5 Y12.5 F100 ; Left side
M5 ; Laser off