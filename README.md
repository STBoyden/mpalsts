# Multiplatform Ambient Light Sensor Theme Switcher

> [!NOTE]
> If you're viewing this on GitHub, please note that this is a mirror for CI/CD
> purposes only. Please make sure to refer all issues and pull requests to the
> [main repository on Codeberg](https://codeberg.org/STBoyden/mpalsts).
>
> For macOS builds, please see
> [here](https://github.com/STBoyden/mpalsts/actions/workflows/build-macos.yml).

![Window Screenshot](docs/assets/window.png)

A multiplatform GUI application that switches your system's theme based on the
ambient light level around your laptop.

Currently supported platforms:

- MacOS
- Linux

Windows support is planned -- however I do not use Windows myself and do not
have plans to install it for the time being.

## Requirements

- A device with an ambient light sensor: Framework 13/16, MacBook Pro, etc.
- A device running on one of the above supported platforms.

## Acknowledgements

The idea for this project came from wanting to port
[DarkModeBuddy](https://github.com/insidegui/DarkModeBuddy) to Linux. The sensor
code for MacOS is a port of DarkModeBuddy's Objective-C code to Rust. The
interface is also basically a straight rip, with some minor modifications.
