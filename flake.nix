{
  description = "Source code of maddie.wtf";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    cargo2nix.url = "github:cargo2nix/cargo2nix/main";
    cargo2nix.inputs.flake-utils.follows = "flake-utils";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";

    onehalf.url = "github:sonph/onehalf/master";
    onehalf.flake = false;
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , onehalf
    , ...
    } @ inputs:
    let
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [
          inputs.cargo2nix.overlays.default
          inputs.fenix.overlays.default

          (final: prev: {
            cargo2nix = inputs.cargo2nix.packages.${system}.default;

            rust-toolchain =
              let
                stableFor = target: target.fromToolchainFile {
                  file = ./rust-toolchain.toml;
                  sha256 = "sha256-3jVIIf5XPnUU1CRaTyAiO0XHVbJl12MSx3eucTXCjtE=";
                };

                rustfmt = final.fenix.latest.rustfmt;
              in
              final.fenix.combine [
                rustfmt
                (stableFor final.fenix)
              ];

            iosevka = (prev.iosevka.override {
              set = "MaddieWtf";
              privateBuildPlan = ''
                [buildPlans.IosevkaMaddieWtf]
                family = "Iosevka MaddieWtf"
                spacing = "normal"
                serifs = "sans"
                noCvSs = true
                exportGlyphNames = false
                webfontFormats = ["woff2"]

                [buildPlans.IosevkaMaddieWtf.ligations]
                enables = [
                  "center-ops",
                  "center-op-trigger-plus-minus-r",
                  "center-op-trigger-equal-l",
                  "center-op-trigger-equal-r",
                  "center-op-trigger-bar-l",
                  "center-op-trigger-bar-r",
                  "center-op-trigger-angle-inside",
                  "center-op-trigger-angle-outside",
                  "center-op-influence-dot",
                  "center-op-influence-colon",
                  "arrow-l",
                  "arrow-r",
                  "counter-arrow-l",
                  "counter-arrow-r",
                  "trig",
                  "eqeqeq",
                  "eqeq",
                  "lteq",
                  "gteq",
                  "exeqeqeq",
                  "exeqeq",
                  "exeq",
                  "eqslasheq",
                  "slasheq",
                  "ltgt-diamond",
                  "ltgt-slash-tag",
                  "slash-asterisk",
                  "kern-dotty",
                  "kern-bars",
                  "logic",
                  "llggeq",
                  "html-comment",
                  "tilde-tilde",
                  "tilde-tilde-tilde",
                  "plus-plus",
                  "plus-plus-plus",
                  "hash-hash",
                  "hash-hash-hash",
                ]
                disables = [
                  "center-op-trigger-plus-minus-l",
                  "arrow-lr",
                  "eqlt",
                  "lteq-separate",
                  "eqlt-separate",
                  "gteq-separate",
                  "eqexeq",
                  "eqexeq-dl",
                  "tildeeq",
                  "ltgt-ne",
                  "ltgt-diamond-tag",
                  "brst",
                  "llgg",
                  "colon-greater-as-colon-arrow",
                  "brace-bar",
                  "brack-bar",
                  "minus-minus",
                  "minus-minus-minus",
                  "underscore-underscore",
                  "underscore-underscore-underscore",
                ]

                [buildPlans.IosevkaMaddieWtf.weights.Regular]
                shape = 400
                menu = 400
                css = 400

                [buildPlans.IosevkaMaddieWtf.widths.Normal]
                shape = 500
                menu = 5
                css = "normal"

                [buildPlans.IosevkaMaddieWtf.slopes.Upright]
                angle = 0
                shape = "upright"
                menu = "upright"
                css = "normal"
              '';
            }).overrideAttrs (old: {
              buildPhase = ''
                export HOME=$TMPDIR
                runHook preBuild
                npm run build --no-update-notifier --targets woff2::$pname -- --jCmd=$NIX_BUILD_CORES --verbose=9
                runHook postBuild
              '';

              installPhase = ''
                runHook preInstall
                fontdir="$out/share/fonts/WOFF2"
                install -d "$fontdir"
                install "dist/$pname/WOFF2"/* "$fontdir"
                runHook postInstall
              '';
            });
          })
        ];
      };

      supportedSystems = with flake-utils.lib.system; [
        aarch64-darwin
        aarch64-linux
        x86_64-darwin
        x86_64-linux
      ];

      inherit (flake-utils.lib) eachSystem;
    in
    eachSystem supportedSystems (system:
    let
      pkgs = pkgsFor system;

      rustPkgs = pkgs.rustBuilder.makePackageSet {
        packageFun = import ./Cargo.nix;
        rustToolchain = pkgs.rust-toolchain;

        packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
          (pkgs.rustBuilder.rustLib.makeOverride {
            name = "maddie-wtf";
            overrideAttrs = drv: {
              COMMIT_HASH = self.rev or self.dirtyShortRev;
            };
          })
        ];
      };

      inherit (pkgs.lib) optionals;
    in
    rec
    {
      packages = rec {
        default = maddie-wtf;
        maddie-wtf = (rustPkgs.workspace.maddie-wtf { }).bin;

        maddie-wtf-static = pkgs.stdenv.mkDerivation {
          name = "maddie-wtf-static";
          srcs = [
            ./static
            "${pkgs.iosevka}/share/fonts/WOFF2"
          ];
          sourceRoot = ".";

          phases = [ "unpackPhase" "installPhase" ];

          installPhase = ''
            mkdir -p $out
            cp -vrf static/* $out
            cp -vrf WOFF2/IosevkaMaddieWtf-Regular.woff2 $out/iosevka-regular.woff2
          '';
        };
      };

      apps = {
        maddie-wtf = flake-utils.lib.mkApp {
          drv = packages.maddie-wtf;
        };
      };

      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo2nix
          convco
          nixpkgs-fmt
          rust-toolchain
          lld

          libiconv
        ] ++ (optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
          CoreServices
        ]));

        THEMES_PATH = "${onehalf}/sublimetext";
        STATIC_PATH = packages.maddie-wtf-static;
      };

      formatter = pkgs.nixpkgs-fmt;
    });
}
