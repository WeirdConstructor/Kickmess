0.2.2 (unreleased)
==================

* Incompatible Change: Click value is limited to the first 1/4th of the
sine phase. This makes a Click value of 1.0 more meaningful, as "full click".
* Change: Improved knob labels and help texts.
* Change: Improved explanation of F1 help mode.
* Change: Mouse button release now does some UI actions directly, instead on
mouse button down.
* Change: Make UI elements gray out when not active.
* Change: Make some UI elements smaller (knobs, text labels).
* Change: Added container titles.
* Change: Added main level and moved the original gain back to the
sin/noise oscillator.
* Change: Presets are now saved as serialized data.
* Change: The Shift key does now also work while dragging (once only, and will
only enable but not disable fine drag mode).
* Change: Value labels are centered now.
* Change: Hide parameters that are not useful to be automateable.
* Feature: Added midi channel selection.
* Feature: Added a filter implementation for filtering the output.
* Feature: Added saw/tri/square oscillator with unison/detune.
* Feature: Added FM oscillator.
* Bugfix: Keys did not work in Ardour/Zrythm and other DAWs.
* Bugfix: The string min/max represenation for the VST parameters were fixed.
* Bugfix: The frequency envelope view in the UI was inverted
before the frequencies were inverted.
* Bugfix: Middle click to restore default value did not restore the default
value, and it did not update the DSP parameter values properly.
* Bugfix: Fixed output range of noise oscillator.
* Bugfix: Escape is a bad keybinding, because in some (most?) hosts
it also closes the window. So right mouse button also exits the input
value mode.
* Bugfix: UI container sizes were not correct.
* Bugfix: Parameter smoothing did not work correctly!

0.2.1 (2021-01-09)
==================

* Feature: Added mouse wheel scrolling.
* Change: Made the coarse adjustment area cover more area.
* Change: Move the fine adjustment area to the knob label.
* Change: Gain only goes up to 2.0 now, to make it
less easy to destroy your speakers.
* Change: Set a fixed scale factor for now until I know
how to deal with system scale factors. Which lead to a differnt UI offset issue.
* Bugfix: Default fine step was too small to feel nice.
* Bugfix: Removed some debug prints.
* Bugfix: A bug in baseview caused the resizing / UI offset issues.

0.1.0 (2021-01-08)
==================
Initial release on GitHub.

* Finished base functionality of this synthesizer.
