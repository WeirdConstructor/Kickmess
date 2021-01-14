# Kickmess - A Kick Drum Synthesizer

This project ports the easy to use and good sounding _Kicker_ plugin from LMMS
to a reusable audio plugin format on Linux. The DSP code has been ported and
also slightly changed, that means an accurate reimplementation of _Kicker_
is out of scope.

Currently this crate generates a VST (Version 2.4) plugin. Eventually this will
be changed to LV2, once rust-lv2 does not require the _inPlaceBroken_ feature
anymore.

Support for other platforms (Windows or MacOS) is currently out of scope,
but depending on the amount of code I might be okay to maintain
contributions.

<a href="http://m8geil.de/data/git/kickmess/res/screenshot1.png">
<img align="left" width="811" height="556" src="http://m8geil.de/data/git/kickmess/res/screenshot1.png">
</a>

## State of Development

This project is still (2021-01-08) under development and considered
useable for what it is. There are features missing, which might come
in future.

Make sure to follow [Weird Constructors Mastodon
account](https://mastodon.online/web/accounts/150371) or the releases of this
project to be notified once I release a beta or stable release.

## Building & Installing

    cargo build --release

    cp target/release/libkickmessvst.so ~/.vst/

## Running the development GUI example

For development the GUI can be executed without any DSP code running
in the background:

    cargo run --release --example gui

## TODO / Features

* A few presets to document a few sweet spots
* Less blurry text (needs improvements in femtovg library/crate)
* Modulation (AHDSR envelope, LFO)
* A second oscillator with sawtooth/square waveforms
* A high/low/band pass filter(s) with resonance

## Known Bugs

* The ones you encounter and create as issues on GitHub.

## Tested Hosts and Systems

| Host | OS | CPU | GPU | WM | State | Date Tested |
|----|-----|-----|----|------|-------|-------------|
| Carla 2.2.0        | Ubuntu 18.04 GNU/Linux | AMD Ryzen | NVidia (propr. drivers) | Gnome/Default  | ok                    | 2021-01-06 |
| Ardour 6.3         | Ubuntu 18.04 GNU/Linux | AMD Ryzen | NVidia (propr. drivers) | Gnome/Default  | ok                    |            |
| Renoise 3.3        | Win 10                 | Intel     | NVidia                  | Win 10         | ok                    | 2021-01-06 |
| Zrythm Alpha 7.1.1 | Win 10                 | Intel     | NVidia                  | Win 10         | ok, clicks on looping | 2021-01-06 |
| Ardour 6.5         | Win 10                 | Intel     | NVidia                  | Win 10         | ok                    | 2021-01-06 |
| FL Studio 20       | Win 10                 | ?         | NVidia                  | Win 10         | ok                    | 2021-01-14 |

## Support Development

You can support me (and the development of this project) via Liberapay:

<a href="https://liberapay.com/WeirdConstructor/donate"><img alt="Donate using Liberapay" src="https://liberapay.com/assets/widgets/donate.svg"></a>

## License

This project is licensed under the GNU Affero General Public License Version 3 or
later.

The DSP code that was translated from LMMS C++ to Rust and was originally
released under GNU General Public License Version 2 or any later.
The former authors were:

* Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
* Copyright (c) 2014 grejppi <grejppi/at/gmail.com>

The fonts DejaVuSerif.ttf and DejaVuSansMono.ttf under the license:

    Fonts are (c) Bitstream (see below). DejaVu changes are in public domain.
    Glyphs imported from Arev fonts are (c) Tavmjong Bah (see below)

### Why (A)GPL?

Picking a license for my code bothered me for a long time. I read many
discussions about this topic. Read the license explanations. And discussed
this matter with other developers.

First about _why I write code for free_ at all, the reasons are:

- It's my passion to write computer programs. In my free time I can
write the code I want, when I want and the way I want. I can freely
allocate my time and freely choose the projects I want to work on.
- To help a friend or member of my family.
- To solve a problem I have.
- To learn something new.

Those are the reasons why I write code for free. Now the reasons
_why I publish the code_, when I could as well keep it to myself:

- So that it may bring value to users and the free software community.
- Show my work as an artist.
- To get into contact with other developers.
- To exchange knowledge and help other developers.
- And it's a nice change to put some more polish on my private projects.

Most of those reasons don't yet justify (A)GPL. The main point of the (A)GPL, as far
as I understand: The (A)GPL makes sure the software stays free software until
eternity. That the _end user_ of the software always stays in control. That the users
have the means to adapt the software to new platforms or use cases.
Even if the original authors don't maintain the software anymore.
It ultimately prevents _"vendor lock in"_. I really dislike vendor lock in,
especially as developer. Especially as developer I want and need to stay
in control of the computers and software I use.

Another point is, that my work (and the work of any other developer) has a
value. If I give away my work without _any_ strings attached, I effectively
work for free. This compromises the price I (and potentially other developers)
can demand for the skill, workforce and time.

This makes two reasons for me to choose the (A)GPL:

1. I do not want to support vendor lock in scenarios for free.
   I want to prevent those when I have a choice, when I invest my private
   time to bring value to the end users.
2. I don't want to low ball my own (and other developer's) wage and prices
   by giving away the work I spent my scarce private time on with no strings
   attached. I do not want companies to be able to use it in closed source
   projects to drive a vendor lock in scenario.

We can discuss relicensing of my code or project if you are interested in using
it in a closed source project. Bear in mind, that I can only relicense the
parts of the project I wrote. If the project contains GPL code from other
projects and authors, I can't relicense it.
