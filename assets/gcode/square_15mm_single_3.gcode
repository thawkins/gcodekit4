; 15mm square cut test for laser cutter
; Square centered at X20 Y20
; Laser intensity S255
; Single pass

G21 ; Units in millimeters
G90 ; Absolute coordinates
G0 X12.5 Y12.5 F500 ; Move to start point (bottom left corner)
M3 S255 ; Laser on at full intensity

G1 X27.5 Y12.5 F500
G1 X27.5 Y27.5 F500
G1 X12.5 Y27.5 F500
G1 X12.5 Y12.5 F500

M5 ; Laser off off now