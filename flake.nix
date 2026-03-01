{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: 
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    nativeBuildInputs = with pkgs; [ rustup ];
    buildInputs = with pkgs; [  ];
    
    cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    pname = cargoTOML.package.name;
    version = cargoTOML.package.version;
  in {
    devShells.${system}.default = pkgs.mkShell {
      inherit nativeBuildInputs buildInputs;  
      C_INCLUDE_PATH = pkgs.lib.makeIncludePath [ ];
      EDITOR = "nvim";
    };
  };
}
