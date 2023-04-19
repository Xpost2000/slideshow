# slideshow
This is a small slideshow program that I made to practice writing Rust.
As a C/C++ programmer, wrapping my head around lifetimes can really hurt.

## Feature List Summary
It is useful for producing small slideshows relatively quickly without much knowledge.

- Multiple Pages
- Basic Styling (Fonts, Small Markup, Color, Arbitrary Positioning)
   - **Bolding**, _Italics_, __Underline__, ~~Strikethrough~~
- Resolution and aspect ratio independence
   - Will provide blackbars on resolutions with different aspect ratios
- Image drawing
- Slide transitions (3 types)
   - Horizontal slideout/in
   - Vertical slideout/in
   - Fade to color

## Technical Description

This slideshow program is made with Rust, and is feature complete. It is sufficient enough for me
to quickly produce slides whenever I need to make presentations.

It's not very difficult to use, and certainly less skill required than using Powerpoint. Of course it is
also more minimal than a fully-fledged Office program, but this is very quick.

Slides are produced through a custom markup language (complete with a custom parser for this language), which
supports the styling options presented above. This markup language supports specifying page transitions of up
to 4 types

SDL2 and it's family libraries are used for rendering things.

Since this is written in Rust, it should be pretty fault proof thanks to it's memory safety.

## Media

## Compile / Build
This thing depends on sdl2-rs and uses [Libre Baskerville](https://fonts.google.com/specimen/Libre+Baskerville)
as the default font.

Those files are included, and this should be buildable, and runnable with

```
cargo build
cargo run
```

Which I admit is way more convenient than my C projects.

## Usage / Running

Running the program will lead to a blank page It includes a dummy
slideshow that was used for testing which also demonstrates all the
features of the markup language.
