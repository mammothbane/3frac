# Controls

## Camera
Use the mouse to control the camera. Hold right click and drag to change the position of the camera eye (motion
restricted to the surface of a sphere about the origin). Use the scroll wheel to zoom in and out. Hold middle-click
and drag to change the camera's center. Press `Enter` to recenter the camera on the origin.

## Transformations
Press `N` to add a new affine map to the scene (represented by a box). `Shift-N` adds a large box.

`Shift-Backspace` deletes the selected transform. 

### Translation
Left-click to select a transform, and drag to move it around. The `W`, `A`, `S`, and `D` keys control X/Z 
translation (`W`/`S` are Z +/-, `A`/`D` are X +/-). `R` and `F` control Y translation. Hold `Shift` while 
pressing any of these keys to enter fine adjustment mode.

### Rotation
`I`, `J`, `K`, `L`, `U`, and `O` control rotation for the selected transform. `I` and `K` are pitch, `J` 
and `L` are roll, `U` and `O` are yaw. Holding `Shift` also makes these adjustments finer.

Press `Backspace` to reset the orientation of the selected transformation.

### Scaling
Hold `X`, `Y`, or `Z` and scroll to scale the selected box in the given dimension. Hold `Shift` to make
this adjustment finer. `B` scales all dimensions at once.

Note: flips are not supported yet.

## Color
Hold `C` and scroll to change the hue of the selected box. `Shift` makes this adjustment finer.

## Fractal iteration
Use the left and right arrow keys to control fractal iteration depth.

`Tab` toggles wireframes on and off.
