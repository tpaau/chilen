<h1 align="center">Chilen</h1>

Fully offline, blazingly fast music player for your library. Built from the
ground-up in Rust with native support for Linux, macOS and Windows.[^1]

![A screenshot of Chilen](https://github.com/tpaau/chilen/blob/main/showcase/chilen-screenshot.jpg)


## Table of contents
- [Installation](#installation)
- [Features](#features)
- [Documentation](#documentation)
- [Inspiration](#inspiration)
- [Contributing](#contributing)

<a name="installation"></a>
## Installation
You can either build Chilen from source or use one of the official installation methods. If you wish
to create a community build, you can contact me and I will happily list it here in a separate
section!

> [!CAUTION]
> Only git builds are available at this time. Those builds are meant primarily for development
> purposes and testing. I do not recommend you actually daily drive Chilen as your music player
> until a tagged release is published. It won't do anything destructive, but do expect issues!

## Fedora
Enable the [`tpaau/chilen`](https://copr.fedorainfracloud.org/coprs/tpaau/chilen/) Copr repo:
```bash
dnf copr enable tpaau/chilen
```

Then install `chilen-git` on regular Fedora:
```bash
dnf install chilen-git
```

Or on Fedora Atomic:
```bash
rpm-stree install chilen-git
```

## Arch Linux
> [!NOTE]
> This is to be published on the AUR, but as of now account registration there is closed due to
> [supply-chain attacks](https://archlinux.org/news/active-aur-malicious-packages-incident/).

```bash
git clone https://github.com/tpaau/chilen-git-src-pkgbuild
cd chilen-git-src-pkgbuild
sudo makepkg -si
```

## Building from source
Please refer to the
[build guide](https://github.com/tpaau/chilen/blob/main/CONTRIBUTING.md#building).


<a name="features"></a>
## Features
- Fully offline
- Blazingly fast 🚀🦀
- Beautiful and user-friendly interface
- Proper desktop integration
- Support for a wide range of audio codecs
- Modular design


<a name="documentation"></a>
## Documentation

[`chilen_backend`](https://tpaau.github.io/chilen/chilen_backend/) - The thing that manages the music library and audio playback, can be reused with different frontends (there may be a mobile port of Chilen someday maybe!!)

Other projects related to Chilen:
- [`iced_m3`](https://tpaau.github.io/chilen/iced_m3/) - Material Design 3 widget library for [iced](https://iced.rs/)
- [`lrc_rs`](https://github.com/tpaau/lrc_rs) - Robust crate for working with synced lyrics content in the LRC format with support for the A2 extension
- [`m3u8_rs`](https://github.com/tpaau/m3u8_rs) - Crate for working with M3U8 playlist files

Also see the [homepage](https://tpaau.github.io/chilen/).


<a name="inspiration"></a>
## Inspiration
- [Auxio](https://github.com/OxygenCobalt/Auxio): Music player that just works
- [Chocola](https://github.com/sosauce/Chocola): Extremely cute and very materially-expressive music player


<a name="contributing"></a>
## Contributing
See [CONTRIBUTING](https://github.com/tpaau/chilen/blob/main/CONTRIBUTING.md).

---

[^1]: Chilen has only been tested on Linux so far. It may or may not work properly on other systems. I plan on expanding platform support in the near future, so stay tuned.
