G21 ; Set units to mm
G90 ; Absolute positioning
G0 X0 Y0 ; Go to origin
G1 X100 Y0 F500 ; Bottom edge
G1 X100 Y50 F500 ; Right edge
G1 X0 Y50 F500 ; Top edge
G1 X0 Y0 F500 ; Left edge
M30 ; End program