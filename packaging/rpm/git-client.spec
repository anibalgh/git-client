Name: git-client
Version: %{version}
Release: 1%{?dist}
Summary: Git-Client: Native 3-Way Merge & Git Client developed in Rust
License: MIT or Apache-2.0
URL: https://github.com/anibalgh/git-client

%description
Git-Client (rmerge) is a high-performance native Git desktop client and 3-way merge tool developed in Rust.

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/512x512/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/128x128/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/64x64/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/48x48/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/32x32/apps
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/16x16/apps

install -m 755 %{_sourcedir}/rmerge %{buildroot}%{_bindir}/rmerge
install -m 755 %{_sourcedir}/rmerge-gui %{buildroot}%{_bindir}/rmerge-gui
install -m 644 %{_sourcedir}/rmerge.desktop %{buildroot}%{_datadir}/applications/rmerge.desktop
install -m 644 %{_sourcedir}/rmerge_512x512.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/rmerge.png
install -m 644 %{_sourcedir}/rmerge_256x256.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/rmerge.png
install -m 644 %{_sourcedir}/rmerge_128x128.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/rmerge.png
install -m 644 %{_sourcedir}/rmerge_64x64.png %{buildroot}%{_datadir}/icons/hicolor/64x64/apps/rmerge.png
install -m 644 %{_sourcedir}/rmerge_48x48.png %{buildroot}%{_datadir}/icons/hicolor/48x48/apps/rmerge.png
install -m 644 %{_sourcedir}/rmerge_32x32.png %{buildroot}%{_datadir}/icons/hicolor/32x32/apps/rmerge.png
install -m 644 %{_sourcedir}/rmerge_16x16.png %{buildroot}%{_datadir}/icons/hicolor/16x16/apps/rmerge.png

%files
%{_bindir}/rmerge
%{_bindir}/rmerge-gui
%{_datadir}/applications/rmerge.desktop
%{_datadir}/icons/hicolor/*/apps/rmerge.png
