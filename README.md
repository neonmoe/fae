# fungui
This is a work-in-progress GUI crate for making easy-to-code
lightweight GUIs. Main features / ideas of the crate:
- [IMGUI](https://caseymuratori.com/blog_0001)-inspired design
- Lightweight (not a wrapper for another UI library, like GTK or Qt)
- No native controls (because of the above point) -> graphics are
  implemented using OpenGL

## Current status
The crate is still very much in development and the API is changing
all the time. Wouldn't recommend using this crate for anything
currently. However, there is one working example, `counter`.

Here's a roadmap, half to describe the status of the crate, half as a
high level todo list:
- [x] Windows, rendering sprites and text, "graphics backend"
- [-] Mouse stuff
  - [x] Button hovers
  - [-] Button presses (buttons on top of eachother still require handling)
  - [ ] Input field focusing
  - [ ] Text selection
  - [ ] Context menu
- [-] Keyboard stuff
  - [x] Arrows to move around in an editable text field
  - [-] Text deletion by backspace and delete (delete not currently implemented)
  - [ ] Ctrl as a modifier to move/delete by word rather than by char
  - [ ] Ctrl+Z to undo, Ctrl+Z/Y to redo
  - [x] Ctrl+V to paste
  - [ ] Text selection
  - [ ] Ctrl+C to copy selection
  - [ ] Tab to switch focus, Shift+Tab to switch to previous

## License
I wouldn't recommend using this crate at this time, but you're free to
do so under the terms of the [MIT license](LICENSE.md).
