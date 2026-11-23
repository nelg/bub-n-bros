# RPM packages for Fedora and the Mandriva family (OpenMandriva, Mageia).
#
# Three packages, which conflict with each other (install one); build each
# with --define "variant ...":
#   python           (default) the game on the system's Python, with all
#                    the clients, including the Rust client
#   server-portable  the game server as one executable with its own Python
#                    (PyInstaller), and the Rust client
#   portable         only the Rust client, with that server built in: one
#                    file, no Python needed
#
#   rpmbuild -ba packing/rpm/bubbros.spec
#   rpmbuild -ba --define "variant server-portable" packing/rpm/bubbros.spec
#   rpmbuild -ba --define "variant portable" packing/rpm/bubbros.spec
# with the release tarball (Source0) in SOURCES.  The build needs network
# access: cargo downloads the Rust client's dependencies (vendor them
# with "cargo vendor" for offline builders), and pip installs PyInstaller
# for server-portable and portable.

%{!?variant:%global variant python}
%global py3 %{?__python3}%{!?__python3:/usr/bin/python3}
%if "%{variant}" != "python"
# only prebuilt-style programs (the Rust client, the PyInstaller server):
# no debug information to split off
%global debug_package %{nil}
%endif
%global others %(for v in python server-portable portable; do [ $v = %{variant} ] || printf 'bubbros-%%s ' $v; done)

Name:           bubbros-%{variant}
Version:        2.4.0
Release:        1%{?dist}
License:        MIT AND Artistic-2.0
URL:            https://github.com/nelg/bub-n-bros
Source0:        https://github.com/nelg/bub-n-bros/archive/refs/tags/v%{version}/bub-n-bros-%{version}.tar.gz
Provides:       bubbros = %{version}-%{release}
Obsoletes:      bubbros < 2.4.0
Conflicts:      %{others}

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  gzip
BuildRequires:  pkgconfig(python3)
BuildRequires:  pkgconfig(alsa)
BuildRequires:  pkgconfig(gl)
BuildRequires:  pkgconfig(xi)
BuildRequires:  pkgconfig(x11)
BuildRequires:  cargo
BuildRequires:  rust >= 1.75
BuildRequires:  desktop-file-utils
%if 0%{?mdvver}
# OpenMandriva: "python" is Python 3
%global pyprefix python
%else
%global pyprefix python3
%endif

%if "%{variant}" == "python"
Summary:        Bubble Bobble clone for up to 10 players (system Python, all clients)
BuildRequires:  pkgconfig(xext)
BuildRequires:  pkgconfig(xrender)
BuildRequires:  %{pyprefix}-setuptools
Requires:       %{pyprefix}
Requires:       %{pyprefix}-pygame
# optional: the GTK 3 display of the Python client
Suggests:       python3-gobject
Suggests:       gtk3
%else
BuildRequires:  binutils
BuildRequires:  pkgconfig(zlib)
BuildRequires:  %{pyprefix}-pip
%endif
%if "%{variant}" == "server-portable"
Summary:        Bubble Bobble clone for up to 10 players (server with its own Python)
%endif
%if "%{variant}" == "portable"
Summary:        Bubble Bobble clone for up to 10 players (portable: one Rust client with the server built in)
%endif

%description
The Bub's Brothers is a fast multiplayer game for up to 10 players, on
one computer or over the network, inspired by Bubble Bobble: blow
bubbles to catch the monsters and burst them, collect the bonuses and
survive the levels.  It includes a large set of levels and mini-games.
%if "%{variant}" == "python"

This package runs the game on the system's Python 3: the game server and
web launcher (menu: "Bub-n-Bros server"), the Python client (pygame, GTK
or X) and the Rust client, bubbros-client ("Bub-n-Bros play alone"),
which plays alone with this server.
%endif
%if "%{variant}" == "server-portable"

This package has the game server as one program with its own Python
inside (built with PyInstaller): bubnbros-server, and its web launcher
bubbros (menu: "Bub-n-Bros server"); and the Rust client,
bubbros-client ("Bub-n-Bros play alone").
%endif
%if "%{variant}" == "portable"

This package is only the Rust client, bubbros-client, with the game
server built in: one program that joins games on the network or, from
the menu ("Bub-n-Bros play alone"), starts a game just for you.
%endif

%prep
%autosetup -n bub-n-bros-%{version}

%build
cd bub-n-bros
export CFLAGS="%{optflags}"
PYTHON=%{py3} sh packing/linux/build.sh %{variant}

%install
cd bub-n-bros
PYTHON=%{py3} sh packing/linux/install.sh %{variant} %{buildroot}
for f in %{buildroot}%{_datadir}/applications/*.desktop; do
    desktop-file-validate "$f"
done

%files
%license bub-n-bros/LICENSE.txt bub-n-bros/artistic.txt
%{_bindir}/bubbros-client
%{_datadir}/applications/bubbros-play-alone.desktop
%{_datadir}/pixmaps/bubbros.png
%{_docdir}/bubbros/
%if "%{variant}" != "portable"
%{_bindir}/bubbros
%{_bindir}/bubnbros-server
%{_datadir}/applications/bubbros.desktop
%{_mandir}/man1/bubbros.1*
%{_mandir}/man1/bubnbros-server.1*
%endif
%if "%{variant}" == "python"
%{_bindir}/bubbros-client-python
%{_datadir}/bubbros/
%{_prefix}/lib/bubbros/
%{_mandir}/man1/bubbros-client-python.1*
%endif

%changelog
* Tue Nov 24 2026 Glen Ogilvie <nelg@users.noreply.github.com> - 2.4.0-1
- Python 3 port, full screen, Rust client with play alone mode
- three packages: python, server-portable, portable
