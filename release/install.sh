#!/bin/bash
# Install a verified local archive or download a platform release.
# Compatible with stock macOS bash and Windows MSYS/MINGW/Cygwin bash.
set -euo pipefail

usage() {
  echo 'Usage: install.sh [--version TAG] [--help]'
  echo '       install.sh ARCHIVE SHA256 INSTALL_DIR'
  echo 'Before installation, verify the downloaded artifact with a current GitHub CLI:'
  echo '  gh attestation verify ARTIFACT --repo genomoncology/biomcp --signer-workflow genomoncology/biomcp/.github/workflows/release.yml --source-ref refs/tags/vX.Y.Z --source-digest COMMIT'
  echo 'Use the release tag commit for COMMIT. Stop if verification fails.'
  echo 'The installer also requires the archive SHA-256 checksum.'
}

fail() { echo "$*" >&2; exit 1; }

download() {
  if command -v curl >/dev/null 2>&1; then
    set -- --proto '=https' --proto-redir '=https' -fsSL --connect-timeout 10 --max-time 120 "$1" -o "$2"
    if [ -n "${CURL_CA_BUNDLE:-}" ]; then
      set -- --cacert "$CURL_CA_BUNDLE" "$@"
    fi
    curl "$@"
  elif command -v wget >/dev/null 2>&1; then
    # GNU wget's --https-only governs recursive links. Check each redirect
    # ourselves because its ordinary downloader may otherwise follow HTTP.
    local url=$1 header next attempt origin
    for attempt in {1..10}; do
      [[ "$url" = https://* ]] || fail 'Download redirect must use HTTPS'
      if wget --https-only --max-redirect=0 --server-response --timeout=30 --tries=1 -qO "$2" "$url" 2> "$scratch/headers"; then
        return 0
      fi
      next=''
      while IFS= read -r header; do
        header=${header%$'\r'}
        if [[ "$header" =~ ^[[:space:]]*[Ll][Oo][Cc][Aa][Tt][Ii][Oo][Nn]:[[:space:]]*(.*)$ ]]; then
          next=${BASH_REMATCH[1]}
        fi
      done < "$scratch/headers"
      [ -n "$next" ] || return 1
      # Resolve ordinary relative Location values while keeping the original
      # HTTPS authority. Never hand wget a URL with a downgraded protocol.
      case "$next" in
        https://*) url=$next ;;
        //*) url="https:$next" ;;
        /*) origin=${url#https://}; origin=${origin%%/*}; url="https://$origin$next" ;;
        *:*) fail 'Download redirect must use HTTPS' ;;
        *) url=${url%%\?*}; url=${url%%\#*}; url="${url%/*}/$next" ;;
      esac
    done
    fail 'Too many download redirects'
  else
    fail 'curl or wget is required to download biomcp'
  fi
}

checksum() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1"
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$1" | sed 's/^.*= //'
  else
    fail 'A SHA-256 tool is required'
  fi
}

# Accept one digest, optionally followed by precisely this asset's filename.
read_checksum() {
  local line records=0
  expected=''
  while IFS= read -r line || [ -n "$line" ]; do
    line=${line%$'\r'}
    [[ "$line" =~ ^[[:space:]]*$ ]] && continue
    records=$((records + 1))
    [ "$records" -eq 1 ] || fail 'Invalid checksum: expected one record'
    if [[ "$line" =~ ^([[:xdigit:]]{64})$ ]]; then
      expected=${BASH_REMATCH[1]}
    elif [[ "$line" =~ ^([[:xdigit:]]{64})[[:space:]]+\*?([^[:space:]]+)$ ]] && [ "${BASH_REMATCH[2]}" = "$asset" ]; then
      expected=${BASH_REMATCH[1]}
    else
      fail 'Invalid checksum record or filename'
    fi
  done < "$1"
  [ "$records" -eq 1 ] || fail 'Missing checksum record'
}

scratch=$(mktemp -d)
stage=''
cleanup() { rm -rf "$scratch"; [ -z "$stage" ] || rm -f "$stage"; }
trap cleanup EXIT
install_dir=${BIOMCP_INSTALL_DIR:-}
binary=biomcp
if [ "$#" -eq 3 ] && [[ "$1" != -* ]]; then
  archive=$1
  expected=$2
  install_dir=$3
else
  version=${BIOMCP_VERSION:-latest}
  while [ "$#" -gt 0 ]; do
    case "$1" in
      -h|--help) usage; exit 0 ;;
      -V|--version)
        [ "$#" -ge 2 ] || fail '--version requires a value'
        version=$2; shift 2 ;;
      *) usage >&2; fail "Unknown argument: $1" ;;
    esac
  done
  if [ -z "$install_dir" ]; then
    [ -n "${HOME:-}" ] || fail 'HOME is unset or empty; set BIOMCP_INSTALL_DIR explicitly'
    install_dir=$HOME/.local/bin
  fi
  [[ "$version" =~ ^[[:alnum:]][[:alnum:]._-]*$ ]] || fail 'Invalid release version'
  system=$(uname -s)
  machine=$(uname -m)
  case "$system" in
    Linux) os=linux ;;
    Darwin) os=darwin ;;
    MINGW*|MSYS*|CYGWIN*) os=windows ;;
    *) fail "Unsupported OS: $system" ;;
  esac
  case "$machine" in
    x86_64|amd64) arch=x86_64 ;;
    arm64|aarch64) arch=arm64 ;;
    *) fail "Unsupported architecture: $machine" ;;
  esac
  suffix=tar.gz
  if [ "$os" = windows ]; then
    [ "$arch" = x86_64 ] || fail "Unsupported platform: $os/$arch"
    suffix=zip
    binary=biomcp.exe
  fi
  asset="biomcp-$os-$arch.$suffix"
  base=${BIOMCP_RELEASE_BASE_URL:-https://github.com/${BIOMCP_GITHUB_REPO:-genomoncology/biomcp}/releases}
  [[ "$base" = https://* ]] || fail 'Release base URL must use HTTPS'
  if [ "$version" = latest ]; then
    url="$base/latest/download/$asset"
  else
    url="$base/download/v${version#v}/$asset"
  fi
  archive="$scratch/$asset"
  download "$url" "$archive" || fail 'Could not download release archive'
  download "$url.sha256" "$scratch/checksum" || fail 'Could not download release checksum; refusing unverified installation'
  read_checksum "$scratch/checksum"
fi
[[ "$expected" =~ ^[[:xdigit:]]{64}$ ]] || fail 'Invalid checksum digest'
actual=$(checksum "$archive")
actual=${actual%% *}
expected=$(printf '%s' "$expected" | tr '[:upper:]' '[:lower:]')
[ "$actual" = "$expected" ] || fail 'Archive checksum mismatch'

# Accept the current notice-bearing layout and the legacy binary-only layout.
# Read only the regular executable. Never extract archive paths.
archive_members() {
  local member executable=$2 binary_seen=0 license_seen=0 notices_seen=0
  while IFS= read -r member; do
    case "$member" in
      "$executable")
        [ "$binary_seen" -eq 0 ] || fail 'Duplicate archive executable'
        binary_seen=1 ;;
      LICENSE)
        [ "$license_seen" -eq 0 ] || fail 'Duplicate archive license'
        license_seen=1 ;;
      THIRD-PARTY-NOTICES.txt)
        [ "$notices_seen" -eq 0 ] || fail 'Duplicate archive notices'
        notices_seen=1 ;;
      *) fail 'Unexpected archive contents' ;;
    esac
  done <<< "$1"
  [ "$binary_seen" -eq 1 ] || fail 'Missing archive executable'
  [ "$license_seen" -eq "$notices_seen" ] || fail 'Incomplete archive notices'
}
case "$archive" in
  *.zip)
    binary=biomcp.exe
    command -v unzip >/dev/null 2>&1 || fail 'unzip is required to install on Windows shells'
    archive_members "$(unzip -Z -1 "$archive")" biomcp.exe
    unzip -p "$archive" biomcp.exe > "$scratch/executable" ;;
  *)
    archive_members "$(tar -tzf "$archive")" biomcp
    [[ "$(tar -tvzf "$archive" biomcp)" = -* ]] || fail 'Archive executable is not a regular file'
    tar -xOzf "$archive" biomcp > "$scratch/executable" ;;
esac
[ -s "$scratch/executable" ] || fail 'Archive executable is empty'
# Strip directory suffix aliases before -L, which follows a trailing slash or /.
while :; do
  case "$install_dir" in
    /) break ;;
    */) install_dir=${install_dir%/} ;;
    */.) install_dir=${install_dir%/.}; install_dir=${install_dir:-/} ;;
    *) break ;;
  esac
done
[ ! -L "$install_dir" ] || fail 'Refusing symbolic-link install directory'
[ ! -L "$install_dir/$binary" ] || fail 'Refusing symbolic-link executable'
[ ! -e "$install_dir/$binary" ] || [ -f "$install_dir/$binary" ] || fail 'Install target is not a regular file'
mkdir -p "$install_dir"
stage=$(mktemp "$install_dir/.biomcp-install.XXXXXX")
cp "$scratch/executable" "$stage"
chmod 755 "$stage"
mv -f "$stage" "$install_dir/$binary"
stage=''
echo "Installed biomcp to $install_dir/$binary"
