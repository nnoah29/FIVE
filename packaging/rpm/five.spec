Name:           five
Version:        %{?pkg_version}%{!?pkg_version:0.1.0}
Release:        1%{?dist}
Summary:        FIVE, a fast native file manager
License:        MIT
URL:            https://github.com/nnoah29/FIVE
BuildArch:      x86_64
Requires:       gtk4 >= 4.16
Requires:       pango >= 1.56
Recommends:     poppler-utils
%global debug_package %{nil}
%global __strip /bin/true

%description
Folder cards, list and grid views, previews, bulk rename, zip, duplicates
and an automation rules engine. Built with Rust and GTK 4.

%install
%{_sourcedir}/stage.sh %{buildroot} %{_prefix}

%files
%{_bindir}/five-gtk
%license %{_datadir}/licenses/five/LICENSE
%{_datadir}/applications/dev.five.files.desktop
%{_datadir}/xdg-desktop-portal/portals/five.portal
%{_datadir}/dbus-1/services/org.freedesktop.impl.portal.desktop.five.service
%{_datadir}/icons/hicolor/scalable/apps/dev.five.files.svg
