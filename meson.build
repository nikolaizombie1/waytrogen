project(
    'waytrogen',
    'rust',
    version: '0.9.6',
    license: 'Unlicence',
    meson_version: '>= 1.2.0',
    default_options: ['rust_std=2021'],
)

gnome = import('gnome')
i18n = import('i18n')

dependency('sqlite3', version: '>= 3.42')
dependency('gtk4', version: '>= 4.12')
dependency('gio-2.0', version: '>= 2.78')
dependency('glib-2.0', version: '>= 2.78')

subdir('src')
subdir('po')


install_data('waytrogen.desktop', install_dir: get_option('prefix') / get_option('datadir') / 'applications')
install_data('README-Assets/WaytrogenLogo.svg', rename: 'waytrogen.svg', install_dir: get_option('prefix') / get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps')
