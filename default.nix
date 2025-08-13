{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
}:

rustPlatform.buildRustPackage (_finalAttrs: {
  pname = "proxy-filter-cli";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoHash = "sha256-iKcJn3dNEWA/ebn23vpHnIm1do3hlooK49gV3BtPtQE=";

  buildAndTestSubdir = "proxy-filter-cli";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl ];

  meta = with lib; {
    description = "App for parsing, filtering and exporting proxy configurations in various formats";
    homepage = "https://github.com/AtaraxiaSjel/proxy-processor";
    license = licenses.mit;
    maintainers = [ ataraxiasjel ];
  };
})
