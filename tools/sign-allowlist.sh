#!/usr/bin/env bash
# Sign the reviewed list, kav/allowlist.toml, after changing it.
#
# usage: tools/sign-allowlist.sh [key]
#   key   the key that signs, by its name, its address or its fingerprint; without one,
#         the only secret key gpg has
#
# It leaves two files beside the list, and both go into git with it:
#   kav/allowlist.toml.sig   the signature of the list exactly as it is now
#   kav/allowlist.gpg        the public half of the key, which kav checks the signature with
#
# kav takes no package from a list that holds packages and whose signature is not good,
# and the kav package does not build with one. Any change to the list, a comment too,
# needs it signed again.
#
# The key is yours and is kept away from this PC, on a USB stick for one. It is made once:
#   export GNUPGHOME=/run/media/$USER/<stick>/kavat-key
#   mkdir -m 700 -p "$GNUPGHOME"
#   gpg --quick-generate-key "Willwitcher-OS Kavat Edition" ed25519 sign never
# and every time after, with the stick in:
#   GNUPGHOME=/run/media/$USER/<stick>/kavat-key tools/sign-allowlist.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
list="$root/kav/allowlist.toml"

key="${1:-}"
if [[ -z $key ]]; then
  mapfile -t keys < <(gpg --batch --with-colons --list-secret-keys | awk -F: '$1 == "sec" { getline; print $10 }')
  [[ ${#keys[@]} -eq 1 ]] || { echo "gpg has ${#keys[@]} secret keys here: name the one that signs, or point GNUPGHOME at where it is" >&2; exit 1; }
  key="${keys[0]}"
fi

gpg --yes --local-user "$key" --output "$list.sig" --detach-sign "$list"
gpg --batch --yes --output "${list%.toml}.gpg" --export "$key"
gpgv --quiet --keyring "${list%.toml}.gpg" "$list.sig" "$list"
echo "kav/allowlist.toml is signed by $key."
