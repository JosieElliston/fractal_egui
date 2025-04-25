# TODO

## actual todo

- learn perturbation theory
- think about thread allocation and workgroup size and stuff
- some needs_update doesn't require regening everything
- subsamples

## requirements

- nice recording
- auto zoom?
- saved positions? (fractal, center, radius)
- mandelbrot, julia, metabrot
- main view must be able to have nothing obstructing it
- the path that z goes
- ui for changing the coloring
- mouse scroll zoom / trackpad pinch zoom
- resizing the window should preserve center and scale

settings

- open with CMD + ,
- textbox entry for main fractal parameters
    - mandelbrot: z0
    - julia: c0
    - metabrot: none
- coloring
- reset camera button / bind

## windows

so we have a main view and little windows for secondary views.
eg mandelbrot on main, unzoomed mandelbrot window, metabrot window with dot for z0.

- should be able to minimize
- should be able to click on metabrot window to change selected z0
- but also have something to lock it to a curve
    - eg z0.real = 0 or on circle or z0.real = a
- right click > set as main window
- maybe disable title_bar

## colorings

- derivative?

## screenshot editor

- when it's open, highlight the bounds on the main(?) screen
