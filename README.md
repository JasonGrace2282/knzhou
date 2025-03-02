# Knzhou
A tool to keep [Kevin Zhou's](https://knzhou.github.io/) handouts up to date.

## Usage
Navigate to the directory of choice, and run
```
knzhou update
```
to install all handouts. To only install/update specific ones, do
```
knzhou update E3     # install E3
knzhou update E3Sol  # install E3 solutions
```

## Installation
Install [Rustup](https://www.rust-lang.org/tools/install), then run
```
rustup toolchain install 1.85  # or higher
git clone https://github.com/JasonGrace2282/knzhou.git
cargo install --path knzhou
```
