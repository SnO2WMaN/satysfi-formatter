{
  pkgs,
  satysfi-formatter ? pkgs.satysfi-formatter,
  ...
}: (
  pkgs.writers.writeBashBin
  "satysfi-fmt-write-each"
  ''
    for f in "$@";
      do ${satysfi-formatter}/bin/satysfi-fmt --write "$f"
    done
  ''
)
