I have been working on a new multiplatform tool for Driving Laser engravers and CNCs called gcodekit4, this started as an exercise in building a complex desktop application using AI, and I belive that has largely been successfull. 

You can find both source code and binary releases at: https://github.com/thawkins/gcodekit4 

The application is built in Rust using the slint GUI tool, it is multiplatform, running on Windows, Linux and MacOS. 

I have tested it on my own Laser Engraver and it works fine. Its able to engrave both vector and bitmap images. It also has built in toolpath generation for Tabbed boxes and Jigsaw puzzles. 

The tool is in alpha status, most of it works, the are bits that dont, there are incomplete sections, but I wanted to get feedback to allow me to prioritize what to do next. 

The UI framework (SLINT) is mostly designed for mobile ad embedded UIs, but it is evolving desktop facilities. 

There is a built in "designer" a simple CAD/CAM system which is functional, can generate gcode, and load and save design files. 

you can find an early User manual in docs/USER.md. 

Some Caveats.

1. The app is using a dark theme, I have the ability to switch themes, but its still being worked on. 
2. The app currently works in millimeters, i plan to have it switchable, internaly it is working in floating point mm values. the internal "world" is +- 1Meter in each direction.
3. there a number of UI bugs that are known about:
    a) keyboard shortcuts dont line up in the menus
    b) tooltips are sometimes displayed under ui controls. 
    c) before you can use the gcode editor, you have to click into it with the mouse, there is a focus issue in Slint. 

im looking for all and any feedback, please create issues on github, bugs, feature requests all will be gratefully welcomed, and I will try to keep up with things. 
