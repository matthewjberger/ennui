# ennui interface rules

These rules apply to every screen and flow of every ennui app. Each app's `CLAUDE.md` imports this file; an app keeps the design of its own screens, its theme tokens and its text sizes in its own interface file.

## What an interface is for

- Judge every screen by how many presses it takes to do the common thing, and whether the user sees what they need at that moment. The common thing is one press wherever it can be.
- Hints and prompts never cover what the user is working on. They are small, sit low or at an edge, show one at a time, and go as soon as the user does the thing or after a few seconds.

## Layout

- Every screen host uses `Host.sizing Tall` with a 1280 by 720 reference, so the interface keeps its proportions and scales with the window height.
- At wide aspects the extra width goes to the sides: pin parts to the left, the center or the right edge, never to fixed x positions.
- Keep content inside a safe area that fits the narrowest supported aspect, with a margin at the top and the bottom. Nothing overflows its panel or the screen at any supported window size: check the tallest column and the widest row against the reference size before you commit a screen.
- Text that can be long wraps inside its panel.

## Theme, shapes and type

- Every screen takes its colors, fonts and sizes from one theme; no screen writes its own copy of a theme value.
- A shaped element (`Style.shaped`, a nine-slice picture that takes the fill color of each state) needs at least 14 pixels of pad, or its edges cover its words.
- One format for each kind of number, used on every screen: a time, a signed difference, a count, a percentage.

## Focus, hover and press

- One highlighted element at a time. Moving the pointer over a button moves the focus to it, so the mouse and the keyboard never show two highlights.
- The focused or hovered element changes its fill and ink and grows a little. The primary button of a screen at rest never looks focused.
- Focus moves to the nearest element in the pressed direction, measured edge to edge.
- When a screen opens, focus starts on its primary action. Menu hosts are modal so this always holds.
- A button that cannot be used now is hidden, not greyed out, unless the user needs to see that it exists, and then it shows its reason.
- A control looks like what it does: arrows beside a value step that value; nothing placed next to a control does something unrelated to it.

## Motion and timing

- Screens enter fast (about 0.2 seconds) with a small overshoot, sliding from the side they belong to. Nothing fades in slowly, and nothing waits for an animation to finish before it takes input: a press during an entrance counts.
- The rows of a list follow each other a few hundredths of a second apart.
- Moments that matter can take longer on purpose, but never block the next action.

## Input

- Every screen works with the mouse and the keyboard, and the user can switch between them at any moment.
- Every action that has a key also has a way to do it with the mouse: a button, a tab, a drag. Key hint strips are not a substitute for buttons.
- Back always returns to the screen that opened the current one and never asks for confirmation. Only actions that destroy something the user cannot get back ask to confirm, on a small card with the safe answer focused.

## Flows

- Settings are read before the window opens, so the first frame is already at the user's window mode and size. There is no splash screen and no account step; the primary action is focused and one press away.
- Teach in use: no tutorial screen and no list of controls. One short prompt at a time, each shown once and gone the moment the user does it.

## Settings

- Every change applies at once and is saved at once; there is no apply button and no unsaved state.
- A reset of all the user's data exists, behind a confirmation, and keeps the settings and the bindings.
- Users can rebind every action on keys and mouse buttons, and a key taken by another action moves rather than being shared by accident.

## Never an unfinished frame

- Every screen stays hidden until its words are bound and its pictures are on the GPU; a bound text that can be empty is hidden by a binding while it is empty.

## Readability

- Never show meaning by color alone: a word, a shape or a sign always carries it too.
- Avoid green against red; pick palettes that stay readable with every common form of color blindness.
- Nothing is smaller than a readable minimum at the reference size, and a text size setting scales all of it.
