all: debug

debug:
	cargo build

release:
	cargo build --release

debug-win:
	cargo build --target x86_64-pc-windows-gnu

release-win:
	cargo build --release --target x86_64-pc-windows-gnu