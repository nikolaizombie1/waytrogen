<h3 align="center">
	<img src="https://github.com/JaKooLit/Telegram-Animated-Emojis/blob/main/Activity/Sparkles.webp" alt="Sparkles" width="38" height="50" />
    $${\color{red}Waytrogen \space \color{lightblue}- \space \color{orange}Wallpaper\space setter\space for\space wayland}$$
	<img src="https://github.com/JaKooLit/Telegram-Animated-Emojis/blob/main/Activity/Sparkles.webp" alt="Sparkles" width="38" height="50" />
</h3>

<div align="center">
<img src="README-Assets/preview.webp" width="100%"/>
</div>

---

<div align="center">
A GUI wallpaper setter for Wayland that is a spiritual successor for the minimalistic wallpaper changer for <code>X11</code> <a href="https://github.com/l3ib/nitrogen">nitrogen</a>. Written purely in the <code>Rust</code> 🦀 programming language.
</div>

## Features
- Recursive and lightning fast file searching
- Can load thousands of wallpapers with ease
- Supports images, GIFs and videos
- Supports external scripts when changing wallpapers
- Can list full wallpaper state in JSON format
- Fully supports:
  - `hyprpaper` (hyprland - png, jpeg, webp, jxl)
  - `swaybg` (sway - png, jpeg, tiff, tga, gif)
  - `mpvpaper` (any video/image format with mpv config)
  - `swww` (jpeg, png, gif, pnm, tga, tiff, webp, bmp, farbfeld with transitions)

## Installation
1. Install required wallpaper changer(s) based on your needs:
    - `hyprpaper` for Hyprland
    - `swaybg` for Sway
    - `mpvpaper` for video support
    - `swww` for transition effects
2. Install `waytrogen`:
    - Arch Linux: Available on [`AUR`](https://aur.archlinux.org/packages/waytrogen)
    - NixOS: Available on [`NUR`](https://github:nikolaizombie1/nur-packages)

## Usage
- Launch via terminal: `waytrogen`
- Restore previous wallpapers: `waytrogen --restore` or `waytrogen -r`
- List current state in JSON: `waytrogen --list` or `waytrogen -l`
- Use external script: `waytrogen --external_script` or `waytrogen -e`
  - Script receives: monitor, wallpaper path, complete state

## Credits
Logo shape from [Inconify Tabler](https://icon-sets.iconify.design/tabler/) atom.
