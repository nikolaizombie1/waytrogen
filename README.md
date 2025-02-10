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
    - NixOS: Available on [`Nixpkgs`](https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=waytrogen)

## Usage
- Launch via terminal: `waytrogen`
- Restore previous wallpapers: `waytrogen --restore` or `waytrogen -r`
- List current state in JSON: `waytrogen --list` or `waytrogen -l`
- Use external script: `waytrogen --external_script` or `waytrogen -e`
  - Script receives: monitor, wallpaper path, complete state
- Cycle to the next wallpaper: `waytrogen --next` or `waytrogen -n` 
  
## Building from source
On NixOS, you can use the provided `flake.nix` to compile and install from source. Just simply clone the repository using:
```bash
git clone https://github.com/nikolaizombie1/waytrogen.git && cd waytrogen
```
For those who would like to build from source on other distributions, the following dependencies are required:
- sqlite3 version 3.42 or greater
- openssl version 3.0 or greater
- gtk4 version 4.12 or greater
- gio-2.0 version 2.78 or greater
- glib-2.0 version 2.78 or greater
- meson version 1.2 or greater
- ninja version 1.10 or greater
- cargo version 1.75 or greater

On Arch use the following command to install the required build dependencies:
```bash
sudo pacman -S gtk4 sqlite openssl glib2 rust meson ninja
```
On Ubuntu use the following command to install the required build dependencies:
```bash
sudo apt install sqlite3 openssl libgtk-4-1 libglib2.0-dev cargo meson ninja-build 
```
Then clone the repository using:
```bash
git clone https://github.com/nikolaizombie1/waytrogen.git && cd waytrogen
```
Configure meson and build using:
```bash
meson setup builddir --prefix=/usr && meson compile -C builddir
```
If you would like to install to your system, use:
```bash
meson install -C builddir
```

## Contribution
All help is welcome and appreciated for `waytrogen`. If you would like to contribute to `waytrogen` follow these steps:
1. Create a fork of `waytrogen` by clicking the `fork` button on the top of the [github](https://github.com/nikolaizombie1/waytrogen) repository.
2. Clone your fork waytrogen:
```bash
git clone https://github.com/YOUR_USERNAME/waytrogen.git && cd waytrogen
```
3. Create a branch who's name describe the changes you would like to do. Please be descriptive, do not use titles such as: `update-readme` or `fix-bug`
```bash
git switch --create descriptive-branch-name main
```
4. Perform the changes you like to do.
   - If you want to add a new language to waytrogen, follow these steps:
	 1. Install `gettext`

	 On Arch Linux use:
	 ```bash
	 sudo pacman -S gettext
	 ```
	 On Ubuntu use:
	 ```bash
	 sudo apt install gettext
	 ```
	 On NixOS gettext is already installed.

	 2. Run `cd po`
	 2. Add the language code you would like to add using a language code from [here](https://www.gnu.org/software/gettext/manual/html_node/index.html) to the `LINGUAS` file. Keep the file ordered alphabetically.
	 4. Create the skeleton `po` file using the following command:
	 ```bash
	 msginit -i waytrogen -o LL.po -l LL_CC.UTF8
	 ```
	 Where `LL` is the language code used in the previous step and `CC` is the country code can be obtained [here](https://www.gnu.org/software/gettext/manual/html_node/Country-Codes.html)

	 5. Modify the skeleton `po` the sections where it says `msgstr ""`
   - If you would like to do code changes, follow these steps:
     1. Install `waytrogen` either from source or from your package manager. This is to install the required schemas. Skip this step if developing on NixOS.
	 2. Install the required development dependencies:

	 On Arch Linux, use:
	 ```bash
	 sudo pacman -S gtk4 sqlite openssl glib2 rust
	 ```
	 On Ubuntu, use:
	 ```bash
	 sudo apt install sqlite3 openssl libgtk-4-1 libglib2.0-dev cargo
	 ```
	 On NixOS, add the following snippet to your `configuration.nix`
	 ```nix
	 programs.direnv = {
	 enable = true;
	 nix-direnv.enable = true;
	 };
	 ```
	 If the nix flake is not loading in properly, run `direnv allow` in the root of the repository to enable the flake.

	 3. Perform the changes you would like to do on `waytrogen`.
	 4. Compile and run `waytrogen` with the new changes using the following based on the distribution:
	  - If on `NixOS` simply run:
	  ```bash
	  nix build && nix run
	  ```
	  - On any other distribution, run:
	  ```bash
	  cargo run --release
	  ```
	  5. Repeat steps 3 and 4 until you are satisfied with your changes.
5. Run `cargo clippy` and make sure there are no suggestions.
6. Format the project using `cargo fmt --all`
7. Create a commit who's message describes your changes in slightly more detail.
8. Go to your fork and create a pull request for `nikolaizombie1:waytrogen` on the main branch. If needed, go into detail what do your changes do in the description of the pull request.

## Credits
Logo shape from [Inconify Tabler](https://icon-sets.iconify.design/tabler/) atom.
