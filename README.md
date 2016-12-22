# hakka [![Build Status](https://travis-ci.org/simon-whitehead/hakka.svg?branch=master)](https://travis-ci.org/simon-whitehead/hakka)
A game that you can't pass simply by playing the game. You have to hack it.

**NOTE: This is currently in "proof-of-concept" stage** and is being actively developed. This early version
is being released to test the waters to see if people find it interesting.

## Requirements
Requires `SDL2`, `SDL2_image`, `SDL2_ttf` and `SDL2_gfx`.

[You can see instructions for installing the SDL2 development libraries here](https://github.com/AngryLawyer/rust-sdl2#sdl20-development-libraries)

## How to play

(There is an in-game `help` command which gives a basic overview of the available commands)

The goal of the training level is simple. Fly the ship up to the finish line:

![screen shot 2016-12-20 at 12 17 17 am](https://cloud.githubusercontent.com/assets/2499070/21314197/bc7765b4-c649-11e6-88e9-89f11b71d704.png)

You can fly it to the finish line with the `Up` arrow. It should be pretty simple..

## Hacking the game

The game includes terminal support for hacking the game. The game runs via an emulated 6502 Microprocessor and
the terminal supports features to interrogate the virtual machine.

### Look at the game code

![screen shot 2016-12-20 at 12 20 06 am](https://cloud.githubusercontent.com/assets/2499070/21314265/2db552a4-c64a-11e6-92b2-332427d09864.png)

### Set breakpoints and step through the code

![screen shot 2016-12-20 at 12 20 22 am](https://cloud.githubusercontent.com/assets/2499070/21314282/48bfbd6e-c64a-11e6-8efd-e8f6d4d024d4.png)

### ..or alter the game memory directly!


## Contributing

I would LOVE contributions. This is currently a single "training" level. I plan on expanding this repository
with new levels in future. If you would like to clean the code up, add documentation, __implement levels__, or anything
else, please do open an Issue and lets discuss it!


## LICENSE
MIT licensed. I hope you learn something from it.

## Credits

The spaceship sprite was created by [MillionthVector and is hosted for free on his/her website](http://millionthvector.blogspot.com.au/p/free-sprites.html). Check it out!

The font is the [FantasqueSansMono font developed by GitHub user belluzj](https://github.com/belluzj/fantasque-sans).
