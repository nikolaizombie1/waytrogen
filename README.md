 <h3 align="center">
	<img src="https://github.com/JaKooLit/Telegram-Animated-Emojis/blob/main/Activity/Sparkles.webp" alt="Sparkles" width="38" height="50" />
    $${\color{red}Waytrogen \space \color{lightblue}- \space \color{orange}Wallpaper\space setter\space for\space wayland}$$
	<img src="https://github.com/JaKooLit/Telegram-Animated-Emojis/blob/main/Activity/Sparkles.webp" alt="Sparkles" width="38" height="50" />
 </h3>

## A GUI wallpaper setter for Wayland that is a spiritual successor for the minimalistic wallpaper changer for `X11` [nitrogen](https://github.com/l3ib/nitrogen). 

## Written purely in the `Rust` 🦀 programming language. Supports `hyprpaper`, `swaybg`, `mpvpaper` and `swww` wallpaper changers.
---
<div align="center">
<img src="README-Assets/preview.webp" width="100%"/>
</div>

---

## Features
- Recursive file searching.
- Lightning fast file searching.
- Can load thousands of wallpapers with ease.
- Fully supports `hyprpaper`, `swaybg`, `mpvpaper` and `swww`.
- Responsive design.
- Supports images, GIFs and videos.
- Supports external scripts when changing wallpapers.
- Can list full wallpaper state in JSON format.

## Installation
1. Install one or more of the following depending on what kind of wallpapers and desktop environment/window manager you are using:
    - `hyprpaper` if using `hyprland` and using only `png`, `jpeg`, `webp` or `jxl` images.
    - `swaybg` if using `sway` and only using `png`, `jpeg`, `tiff`, `tga` or `gif` images.
    - `mpvpaper` if using any kind of video or image format, but requires command line arguments to be passed to `mpv` for additional configuration.
    - `swww` for displaying `jpeg`, `png`, `gif`, `pnm`, `tga`, `tiff`, `webp`, `bmp` or `farbfeld` images and want transitions between images.
2. Install `waytrogen` using:
    - If on Arch Linux, `waytrogen` is on the [`AUR`](https://aur.archlinux.org/packages/waytrogen)
    - If on NixOS, `waytrogen` is on the [`NUR`](https://github:nikolaizombie1/nur-packages)

## Dependencies
- `hyprpaper`, `swaybg`, `mpvpaper` or `swww`
- `ffmpeg`

## Usage
The `waytrogen` command can be used the terminal to launch the application or be launched using an application launcher.

The `waytrogen --restore` or `waytrogen -r` command can be used to restore previously set wallpapers.

The `waytrogen --list` or `waytrogen -l` command prints, to standard output, the current state of wallpaper changers in JSON format 

The `waytrogen --external_script` or `waytrogen -e` command passes an external executable script to `waytrogen`. It executes after a wallpaper has changed and is passed the following arguments:
1. The `monitor` that the wallpaper was set to.
2. The `path` of the wallpaper.
3. The `complete state` of the currently set wallpapers.

## Credits
The logo shape is from [Inconify Tabler](https://icon-sets.iconify.design/tabler/) atom.
