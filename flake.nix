{
  description = "Front-end for chat backends";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";  # Specify the Nixpkgs version
	rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
  flake-utils.lib.eachDefaultSystem (system:
  let
  	overlays = [ 
		(import rust-overlay)
	];
    pkgs = import nixpkgs {
		inherit system overlays;
	};
  in
  {
		devShells = {
			default = pkgs.mkShell.override { stdenv = pkgs.clangStdenv; } {
    		    packages = with pkgs; [
				  rust-bin.nightly.latest.default
				  # rust-bin.stable.latest.default
    		      rust-analyzer

				  cargo-expand

				  nodejs_24

				#   alsa-lib
				  libopus

					at-spi2-atk
    				atkmm
    				cairo
    				gdk-pixbuf
    				glib
    				gtk3
    				harfbuzz
    				librsvg
    				libsoup_3
    				pango
    				# webkitgtk_4_1
    				openssl

				  pkg-config
    		    ];


    		    # RUST_BACKTRACE = "full";
				
				# Wayland
    		    # WINIT_UNIX_BACKEND = "wayland";
    		    
				# X11/Xwayland
				# WINIT_UNIX_BACKEND = "x11";
				# WAYLAND_DISPLAY="";
    		};
		};
	});
}
