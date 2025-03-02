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

### Changing the Config
`knzhou` has a system level config file located at `~/.config/knzhou/knzhou.toml` on Unix.
To get the location of the config file on your system, run `knzhou config get`.

#### Formatting Handout Names
In the `knzhou` config, there is a parameter called `format`. It uses the special
value `{handout}`, which is replaced with the name of the handout. For example,
to name `E3.pdf` as `handout-E3-best.pdf`, set it to the following
```toml
format = "handout-{handout}-best"
```

## Installation
Install [Rustup](https://www.rust-lang.org/tools/install), then run
```
rustup toolchain install 1.85  # or higher
git clone https://github.com/JasonGrace2282/knzhou.git
cargo install --path knzhou
```
