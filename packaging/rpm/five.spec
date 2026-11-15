Name:           five
Version:        %{?pkg_version}%{!?pkg_version:0.1.0}
Release:        1%{?dist}
Summary:        FIVE, a fast native file manager
License:        LicenseRef-Unspecified
URL:            https://github.com/nnoah29/FIVE
BuildArch:      x86_64
Requires:       gtk4 >= 4.16
Requires:       pango >= 1.56
%global debug_package %{nil}
%global __strip /bin/true

%description
Folder cards, list and grid views, previews, bulk rename, zip, duplicates
and an automation rules engine. Built with Rust and GTK 4.

%install
%{_sourcedir}/stage.sh %{buildroot} %{_prefix}

%files
%{_bindir}/five-gtk
%{_datadir}/applications/dev.five.files.desktop
%{_datadir}/icons/hicolor/scalable/apps/dev.five.files.svg
