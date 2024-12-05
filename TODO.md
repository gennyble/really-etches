Keyboard Input
	Nov 30th:
	the main input method right now is spinning the two thumbsticks on
	some kind of game controller, but you can't spin a keyboard key.

	so i was thinking:
	the left stick, horizontal, could be asdf and the right cjkl; and
	you would sort of pulse down the keys to imitate a spin. the timing
	between the presses, your acceleration per se, would determine how
	fast the line is being drawn. a-f is to the right and f-a is to the
	left. j-; is up, ;-j is down.

	Dec 5:
	we have this working! it's pretty clunky, but it's working!

	we should allow changing what keys are used in the gallop. this can
	be in a config file with the characters just in a string. for example,
	horizontal could look like: `HorizontalGallop asdf` and vertical could
	be: `VerticalGallop jkl;`. reversing those characters could allow you
	to invert the direction of the gallop, but that should also be an option.

Images save as GIF and PNG
	gif can have a palette of 4 and so can the png. The png might be able
	to have a depth of 2-bits per pixel which is exciting.

	images should be saved in the underlying resolution and not be dpi
	scaled. images should not include the stylus.

Clearing should be gradual
	It should require multiple button presses, to imitate shaking a thing,
	and with each shake the existing lines should get some fixed amount
	closer to the background colour. maybe they gain 10% of the difference
	between the line and background colour.

	It would be cute if the screen did a side-to-side animation on each
	press as well.