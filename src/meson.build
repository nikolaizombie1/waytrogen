cargo = find_program('cargo')
cargo_env = ['CARGO_HOME=' + meson.project_build_root() / 'cargo-home']
cargo_options = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
cargo_options += ['--target-dir', meson.project_build_root() / 'target']

cargo_features = get_option('cargo_features')

if cargo_features != ''
   cargo_options += ['--features', cargo_features]
endif

cargo_release = custom_target(
    'cargo-build',
    build_by_default: true,
    build_always_stale: true,
    console: true,
    output: 'waytrogen',
    install: true,
    install_dir: join_paths(get_option('prefix'), get_option('bindir')),
    command: [
        cargo,
        'build',
        '--release', cargo_options,
        '&&',
        'cp', 'target' / 'release' / meson.project_name(), '@OUTPUT@' 
    ],
)
