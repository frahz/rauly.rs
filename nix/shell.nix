{pkgs}:
pkgs.mkShell {
  name = "raulyrs";
  nativeBuildInputs = with pkgs; [pkg-config];
  buildInputs = with pkgs; [
    rust-bin.stable.latest.default
    yt-dlp
    openssl
    libopus
  ];
}
