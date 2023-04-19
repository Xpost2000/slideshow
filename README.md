# slideshow

This is a small slideshow program that I made to practice writing Rust.

It is useful for producing small slideshows relatively quickly without much knowledge.

It does the following:
  - Multiple Pages
  - Basic Styling (Fonts, Small Markup, Color, Arbitrary Positioning)
  - Resolution and aspect ratio independence
  - Image drawing
  - Slide transitions (4 types)

It's also kind of convenient for me since I don't have Microsoft Powerpoint so I can use this to make
basically presentable slides.

## Compile / Build
This thing depends on sdl2-rs and uses [Libre Baskerville](https://fonts.google.com/specimen/Libre+Baskerville)
as the default font.

Those files are included, and this should be buildable with

```
cargo build
```

## Usage / Running
Running the program will lead to a blank page
It includes a dummy slideshow that was used for testing but also demonstrates the format of the slides used.
Lifetimes really hurt and I need to get to know them better...
