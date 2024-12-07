(1) Keyboard Input
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

	Dec 7th:
	this is pretty clunky. perhaps the tuning cannot fix it. we should try
	and see if just advancing the cursor on keypress feels alright. the
	direction thing would need work, though. "on keypress" to mean the
	entire press not while down. I want a press to be an action alone and
	not some kind of "smooth draw". although i guess that's what the
	etch-a-sketch would be like? lots to think about.

(2) Mouse Input
	this would be cute. Two mice required :)

	one scroll wheel for each degree of freedom. we can have a little
	sub-window where you can click to identify the mice, "vertical" and
	"horizontal", and then perhaps a setting to switch them.

(3) Images save as GIF and PNG
	gif can have a palette of 4 and so can the png. The png might be able
	to have a depth of 2-bits per pixel which is exciting.

	images should be saved in the underlying resolution and not be dpi
	scaled. images should not include the stylus.

(4) Clearing should be gradual
	It should require multiple button presses, to imitate shaking a thing,
	and with each shake the existing lines should get some fixed amount
	closer to the background colour. maybe they gain 10% of the difference
	between the line and background colour.

	It would be cute if the screen did a side-to-side animation on each
	press as well.

(5) Crash when you try to go below the bottom
	If you're moving down and you hit the bottom edge, we crash.

(6) Lines with a positive slope draw wrong
	When you're diagonaling right and up, the line draws almost-inverted
	and comes out as right and down, but it's more than that. Seemingly,
	the start and end of the slop-line draw with swapped Y coordinates.

	image.rs:98
	this just swaps the problem. i think what we need to do is figure
	out when it gets mad, possibly all the time when the starting Y is
	lower in the image (so higher Y value) than the ending position.
	And then we should special-case that. Or attempt to use a "real"
	line drawing algorithm. Draw larger-than-a-pixel-squares at every
	point is certainly an algorithm, but probably not the most efficient
	one we can use. Modified brashman's for thick lines?

==== DONE? ====

(7) Reduce memory usage with profiling
	resources:
	- A blog post from the rust-analyzer team on memory profiling (Dec. 2020)
	  https://rust-analyzer.github.io/blog/2020/12/04/measuring-memory-usage-in-rust.html
	- Polar Signals blog post on using some jemalloc instrumentation (Oct. 2023)
	  https://www.polarsignals.com/blog/posts/2023/12/20/rust-memory-profiling
	- Quickwit post on using heaptrack. This one first!! (Mar. 2022)
	  https://quickwit.io/blog/memory-inspector-gadget

	currently, on macOS, it seems we use 18MB while running. This seems reasonable
	for a GUI application but it also seems high? All we really have is a window
	with a place to put pixels. If my math isn't terribly wrong, which it might be,
	we have

	DONE
	because i wrote the smallest winit program, none-surface-left-beef, that's only
	job was to make a winit window. It used 11MB. Our surfaces, assuming that
	softbuffer holds only one buffer in memory, was ~5.2MB (~5MB for the 640x480 window
	at a 2x scale, and ~1.2MB for 640x480 (both using u32s for pixels)).