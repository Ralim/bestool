{ lib
, pkgs
, rustPlatform
,
}:
rustPlatform.buildRustPackage {
  name = "bestool";

  src = ./bestool;

  cargoLock = {
    lockFile = ./bestool/Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = with pkgs; [ pkg-config ];
  buildInputs = with pkgs; [ systemd.dev ];

  meta = with lib; {
    description = "";
    homepage = "https://github.com/Ralim/bestool";
    platforms = platforms.linux;
    maintainers = with maintainers; [ shymega ];
    license = licenses.mit;
  };
}
