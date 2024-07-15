# uses the flake
(import (fetchTarball {
    url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    sha256 = "sha256:0m9grvfsbwmvgwaxvdzv6cmyvjnlww004gfxjvcl806ndqaxzy4j";
}) {
    src = ./.;
}).shellNix