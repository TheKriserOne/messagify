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

				  alsa-lib
				  libopus

				  fontconfig

				  openssl
				  pkg-config
    		    ];
				LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
					pkgs.libxkbcommon
					pkgs.wayland
					pkgs.vulkan-loader
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
