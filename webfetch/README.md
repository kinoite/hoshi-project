> [!CAUTION]
> webfetch will be deprecated and will be replaced with the much more performant, faster (dncli)[https://github.com/kinoite/dncli]

# webfetch

webfetch is a headless utility apart of the Hoshi Project which downloads files from the web

webfetch is meant to be a more modular, powerful and fast alternative to the old C-written wget.

## Use

```
webfetch_cli https://github.com/rust-lang/rust-artwork/blob/master/logo/rust-logo-512x512.png -o rust-logo-512x512.png
```

ommiting -o would make you type the output file 

## Installation

Clone this repository and then run:
```
cargo build --release -p webfetch
```
then to move it to your `$PATH`
```
mv ~/hoshi-project/target/release/webfetch /usr/local/bin # you can change /usr/local/bin to any $PATH you prefer
```
