project(
  'kepler-shell',
  'rust',
  version: '0.1.0',
  meson_version: '>= 0.60.0',
  default_options: [
    'warning_level=2',
    'werror=false',
  ],
)

gnome = import('gnome')
i18n = import('i18n')

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
pkgdatadir = datadir / 'kepler-shell'

# Cargo build custom target
cargo = find_program('cargo', required: true)
cargo_release = get_option('buildtype') == 'release'

cargo_args = ['build', '--manifest-path', meson.project_source_root() / 'Cargo.toml']
if cargo_release
  cargo_args += ['--release']
  cargo_target_dir = meson.project_source_root() / 'target' / 'release'
else
  cargo_target_dir = meson.project_source_root() / 'target' / 'debug'
endif

custom_target(
  'cargo-build',
  build_by_default: true,
  output: ['kepler-shell'],
  console: true,
  install: true,
  install_dir: bindir,
  command: [
    cargo,
    cargo_args,
    '&&',
    'cp',
    cargo_target_dir / 'kepler-shell',
    '@OUTPUT@',
  ],
)

# Install data directories
install_data(
  'data/config.default.toml',
  install_dir: pkgdatadir,
)

install_subdir(
  'data/palettes',
  install_dir: pkgdatadir,
)

install_subdir(
  'data/style',
  install_dir: pkgdatadir,
)

# Install GNOME Shell Extension
install_subdir(
  'extension',
  install_dir: datadir / 'gnome-shell' / 'extensions' / 'kepler-shell@antonio.local',
)
