// The files of the current release.  To publish a new release, change
// VERSION and the file list (name: size in bytes, as on the release page).

const VERSION = "2.4.0";
const RELEASE_PAGE = "https://github.com/nelg/bub-n-bros/releases/tag/v" + VERSION;
const DOWNLOAD_BASE = "https://github.com/nelg/bub-n-bros/releases/download/v" + VERSION + "/";

const RELEASE_FILES = {
  "bubbros-portable-x86_64-pc-windows-msvc.zip": 23531086,
  "bubbros-portable-aarch64-apple-darwin.tar.gz": 23471611,
  "bubbros-portable-x86_64-unknown-linux-gnu.tar.gz": 37874952,
  "bubbros-portable_2.4.0-1_amd64-ubuntu-24.04.deb": 21237542,
  "bubbros-portable_2.4.0-1_amd64-debian-trixie.deb": 22107008,
  "bubbros-portable-2.4.0-1.fc45.x86_64.fedora.rpm": 24544887,
  "bubbros-portable-2.4.0-1-omv2590.znver1.openmandriva.rpm": 24467466,

  "bubbros-server-portable-x86_64-pc-windows-msvc.zip": 22948545,
  "bubbros-server-portable-aarch64-apple-darwin.tar.gz": 22697547,
  "bubbros-server-portable-x86_64-unknown-linux-gnu.tar.gz": 37209749,
  "bubbros-server-portable_2.4.0-1_amd64-ubuntu-24.04.deb": 21233036,
  "bubbros-server-portable_2.4.0-1_amd64-debian-trixie.deb": 22100740,
  "bubbros-server-portable-2.4.0-1.fc45.x86_64.fedora.rpm": 24541711,
  "bubbros-server-portable-2.4.0-1-omv2590.znver1.openmandriva.rpm": 24463205,

  "bubbros-python-2.4.0.zip": 12866653,
  "bubbros-python-2.4.0.tar.gz": 12805883,
  "bubbros-python_2.4.0-1_amd64-ubuntu-24.04.deb": 10510464,
  "bubbros-python_2.4.0-1_amd64-debian-trixie.deb": 9922260,
  "bubbros-python-2.4.0-1.fc45.x86_64.fedora.rpm": 10868183,
  "bubbros-python-2.4.0-1-omv2590.znver1.openmandriva.rpm": 10833173,

  "bubbros-python-dbgsym_2.4.0-1_amd64-debian-trixie.deb": 38564,
  "bubbros-python-debuginfo-2.4.0-1.fc45.x86_64.fedora.rpm": 3677241,
  "bubbros-python-debugsource-2.4.0-1.fc45.x86_64.fedora.rpm": 47913,
  "bubbros-python-debuginfo-2.4.0-1-omv2590.znver1.openmandriva.rpm": 38509,
  "bubbros-python-debugsource-2.4.0-1-omv2590.znver1.openmandriva.rpm": 16157,
};

// What each kind of file is, for the table.
const KINDS = {
  "portable": "Portable: the Rust client with the game server built in. One program, no Python needed. Plays alone or joins games.",
  "server-portable": "Portable server: the game server with its own Python inside, for hosting games others join. Linux packages also include the Rust client.",
  "python": "Python: the game for your own Python 3 (needs pygame). Server, web launcher and Python clients; the Linux packages add the Rust client.",
  "debug": "Debugging information for the python package. Not needed to play.",
};

const SYSTEMS = {
  "windows": "Windows (64-bit)",
  "macos-arm": "macOS, Apple Silicon (M1 and newer)",
  "linux-generic": "Linux, any distribution (x86-64)",
  "ubuntu": "Ubuntu 24.04 and newer",
  "debian": "Debian 13 (trixie)",
  "fedora": "Fedora",
  "openmandriva": "OpenMandriva",
  "any": "Any system with Python 3 and pygame",
};

function describeFile(name) {
  let kind = "python";
  if (/-(dbgsym|debuginfo|debugsource)/.test(name)) kind = "debug";
  else if (name.startsWith("bubbros-server-portable")) kind = "server-portable";
  else if (name.startsWith("bubbros-portable")) kind = "portable";
  let system = "any", format = "";
  if (name.includes("windows")) { system = "windows"; format = "Zip archive"; }
  else if (name.includes("apple-darwin")) { system = "macos-arm"; format = "tar.gz archive"; }
  else if (name.includes("linux-gnu")) { system = "linux-generic"; format = "tar.gz archive"; }
  else if (name.includes("ubuntu")) { system = "ubuntu"; format = "Debian package (.deb)"; }
  else if (name.includes("debian")) { system = "debian"; format = "Debian package (.deb)"; }
  else if (name.includes("fedora")) { system = "fedora"; format = "RPM package"; }
  else if (name.includes("openmandriva")) { system = "openmandriva"; format = "RPM package"; }
  else if (name.endsWith(".zip")) { format = "Zip archive (best for Windows)"; }
  else if (name.endsWith(".tar.gz")) { format = "tar.gz archive (macOS, Linux)"; }
  return { kind, system, format };
}

function humanSize(bytes) {
  if (bytes >= 1e6) return (bytes / 1e6).toFixed(1) + " MB";
  return Math.max(1, Math.round(bytes / 1e3)) + " kB";
}
