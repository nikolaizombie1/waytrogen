project(
    'waytrogen',
    'rust',
    version: 'VERSION',
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

install_data('org.Waytrogen.Waytrogen.gschema.xml', install_dir: join_paths(get_option('prefix'), join_paths(get_option('datadir')), 'glib-2.0', 'schemas'))


install_data('waytrogen.desktop', install_dir: join_paths(get_option('prefix'), join_paths(get_option('datadir')), 'applications'))
install_data('README-Assets/WaytrogenLogo.svg', rename: 'waytrogen.svg', install_dir: join_paths(get_option('prefix'), join_paths(get_option('datadir')), 'icons', 'hicolor', 'scalable', 'apps'))
gnome.post_install(glib_compile_schemas: not get_option('debian_install'), gtk_update_icon_cache: true, update_desktop_database: true)
