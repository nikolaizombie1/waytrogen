project(
    'waytrogen',
    'rust',
    version: '0.9.8',
    license: 'Unlicence',
    meson_version: '>= 1.2.0',
    default_options: ['rust_std=2021'],
)

dependency('sqlite3', version: '>= 3.39')

subdir('src')


install_data('waytrogen.desktop', install_dir: get_option('prefix') / get_option('datadir') / 'applications')
install_data('README-Assets/WaytrogenLogo.svg', rename: 'waytrogen.svg', install_dir: get_option('prefix') / get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps')
