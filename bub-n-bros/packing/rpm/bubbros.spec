# RPM package for Fedora, openSUSE-like systems and the Mandriva family
# (OpenMandriva, Mageia).  Build from a release tarball:
#   spectool -g -R packing/rpm/bubbros.spec   (or download Source0 by hand)
#   rpmbuild -ba packing/rpm/bubbros.spec
# The native client's Rust dependencies are downloaded by cargo during
# the build (vendor them with "cargo vendor" for offline builders).

%global py3 %{?__python3}%{!?__python3:/usr/bin/python3}

Name:           bubbros
Version:        2.4.0
Release:        1%{?dist}
Summary:        Multiplayer Bubble Bobble clone for up to 10 players
License:        MIT AND Artistic-2.0
URL:            https://github.com/nelg/bub-n-bros
Source0:        https://github.com/nelg/bub-n-bros/archive/refs/tags/v%{version}/bub-n-bros-%{version}.tar.gz

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  gzip
BuildRequires:  pkgconfig(python3)
BuildRequires:  pkgconfig(x11)
BuildRequires:  pkgconfig(xext)
BuildRequires:  pkgconfig(xrender)
BuildRequires:  pkgconfig(alsa)
BuildRequires:  pkgconfig(gl)
BuildRequires:  pkgconfig(xi)
BuildRequires:  cargo
BuildRequires:  rust >= 1.75
BuildRequires:  desktop-file-utils
%if 0%{?mdvver}
# OpenMandriva: "python" is Python 3
BuildRequires:  python-setuptools
Requires:       python
Requires:       python-pygame
%else
BuildRequires:  python3-setuptools
Requires:       python3
Requires:       python3-pygame
%endif
# optional: the GTK 3 display of the Python client
Suggests:       python3-gobject
Suggests:       gtk3

%description
The Bub's Brothers is a fast multiplayer game for up to 10 players, on
one computer or over the network, inspired by Bubble Bobble: blow
bubbles to catch the monsters and burst them, collect the bonuses and
survive the levels.  It includes a large set of levels and mini-games.

This package contains the game server and web launcher (bubbros), the
Python client (bubnbros-client) and the native client (bubbros-native),
which can also start a server of its own to play alone.

%prep
%autosetup -n bub-n-bros-%{version}
cd bub-n-bros
# keep the launcher settings in ~/.bubnbros
patch -p1 < packing/arch/bubbros-config-in-home.patch

%build
cd bub-n-bros
export CFLAGS="%{optflags}"
PYTHON=%{py3} sh packing/linux/build.sh

%install
cd bub-n-bros
PYTHON=%{py3} sh packing/linux/install.sh %{buildroot}
desktop-file-validate %{buildroot}%{_datadir}/applications/bubbros.desktop
desktop-file-validate %{buildroot}%{_datadir}/applications/bubbros-play-alone.desktop

%files
%license bub-n-bros/LICENSE.txt bub-n-bros/artistic.txt
%{_bindir}/bubbros
%{_bindir}/bubbros-native
%{_bindir}/bubnbros-client
%{_bindir}/bubnbros-server
%{_datadir}/bubbros/
%{_prefix}/lib/bubbros/
%{_datadir}/applications/bubbros.desktop
%{_datadir}/applications/bubbros-play-alone.desktop
%{_datadir}/pixmaps/bubbros.png
%{_mandir}/man1/bubbros.1*
%{_mandir}/man1/bubnbros-client.1*
%{_mandir}/man1/bubnbros-server.1*
%{_docdir}/bubbros/

%changelog
* Tue Nov 24 2026 Glen Ogilvie <nelg@users.noreply.github.com> - 2.4.0-1
- Python 3 port, full screen, native client with play alone mode
