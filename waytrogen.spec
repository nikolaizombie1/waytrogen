%global debug_package %{nil}

Name:           waytrogen
Version:        1.0.1
Release:        %autorelease
Summary:        A GUI wallpaper changer for wayland

BuildRequires: rust-packaging >= 26
BuildRequires: meson
BuildRequires: ninja-build
BuildRequires: pkgconfig(sqlite3)

SourceLicense:  Unlicense
License:        %{shrink:
    Unlicense AND 
    (MIT OR Apache-2.0) AND NCSA
    (MIT OR Apache-2.0) AND Unicode-3.0
    0BSD OR MIT OR Apache-2.0
    Apache-2.0
    Apache-2.0 AND MIT
    Apache-2.0 OR GPL-2.0-only
    Apache-2.0 OR MIT
    Apache-2.0 OR MIT OR CC0-1.0
    Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT
    BSD-2-Clause
    BSD-2-Clause OR Apache-2.0 OR MIT
    BSD-3-Clause
    BSD-3-Clause OR Apache-2.0
    BSD-3-Clause OR MIT OR Apache-2.0
    BSL-1.0
    CC0-1.0
    CC0-1.0 OR Apache-2.0
    ISC
    MIT
    MIT OR Apache-2.0
    MIT OR Apache-2.0 OR LGPL-2.1-or-later
    MIT OR Apache-2.0 OR Zlib
    MIT OR Zlib OR Apache-2.0
    Unicode-3.0
    Unlicense OR MIT
    Zlib
    Zlib OR Apache-2.0 OR MIT
}

URL:            https://github.com/nikolaizombie1/%{name}
Source:         %{url}/archive/refs/tags/%{version}.tar.gz
Source1:        %{url}/releases/download/%{version}/%{name}-%{version}-vendor.tar.gz

BuildRequires:  cargo-rpm-macros

%description
A GUI wallpaper changer for wayland.

%prep
%autosetup -p1 -a 1
%cargo_prep -v vendor

%build
%meson
%meson_build
%cargo_vendor_manifest
%{cargo_license_summary}
%{cargo_license} > LICENSE.dependencies

%install
%meson_install

%check
%{buildroot}%{_bindir}/%{name} --help 

%files
%license LICENSE
%license LICENSE.dependencies
%doc README.md
%{_bindir}/%{name}
%{_datadir}/applications/%{name}.desktop
%{_datadir}/icons/hicolor/*/apps/%{name}.svg
%{_datadir}/bash-completion/completions/%{name}.bash
%{_datadir}/zsh/vendor_completions/_%{name}
%{_datadir}/fish/vendor_completions.d/%{name}.fish


%changelog
%autochangelog
