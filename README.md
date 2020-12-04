# slideshow

This is a small slideshow program that I made to practice doing some Rust code.

It wasn't meant to practice idiomatic Rust code, it was just to produce something, and
it worked although I'm not very happy with a lot of the things I did and I've got to unC-ify
certain parts of this.

It does the following:
  - Multiple Pages
  - Basic Styling (Fonts, Small Markup, Color, Arbituary Positioning)
  - Resolution Independence
  - Image drawing
  - Slide transitions.

It's also kind of convenient for me since I don't have Microsoft Powerpoint so I can use this to make
basically presentable slides.

This thing depends on sdl2-rs and uses [Libre Baskerville](https://fonts.google.com/specimen/Libre+Baskerville)
as the default font.

It includes a dummy slideshow that was used for testing but also demonstrates the format of the slides used.
Lifetimes really hurt and I need to get to know them better...
