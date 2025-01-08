# Waytrogen
A GUI wallpaper setter for Wayland that is a spiritual successor for the minimalistic wallpaper changer for `X11` [nitrogen](https://github.com/l3ib/nitrogen). Written purely in the `Rust` 🦀 programming language. Supports `hyprpaper`, `swaybg`, `mpvpaper` and `swww` wallpaper changers.
---
---
## Features
- Recursive file searching
- Lightning fast file searching
- Can load thousands of wallpapers with ease
- Fully supports `hyprpaper`, `swaybg`, `mpvpaper` and `swww`.
- Responsive design
- Supports images, GIFs and videos
## Installation
1. Install one or more of the following depending on what kind of wallpapers and desktop environment/window manager you are using:
    - `hyprpaper` if using `hyprland` and using only `png`, `jpeg`, `webp` or `jxl` images.
    - `swaybg` if using `sway` and only using `png`, `jpeg`, `tiff`, `tga` or `gif` images.
    - `mpvpaper` if using any kind of video or image format, but requires command line arguments to be passed to `mpv` for additional configuration.
    - `swww` for displaying `jpeg`, `png`, `gif`, `pnm`, `tga`, `tiff`, `webp`, `bmp` or `farbfeld` images and want transitions between images.
2. Install `waytrogen` using `cargo install waytrogen`.
## Dependencies
- `hyprpaper`, `swaybg`, `mpvpaper` or `swww`
- `ffmpeg`
## Usage
The `waytrogen` command can be used the terminal to launch the application or be launched using an application launcher.

The `waytrogen --restore` command can be used to restore previously set wallpapers.