%global debug_package %{nil}

Name:           chilen-git
Version:        0.1.0
Release:        1.git%{?dist}
Summary:        Fully offline, blazingly fast music player for your library

License:        GPL-3.0-or-later
URL:            https://github.com/tpaau/chilen
Source0:        https://github.com/tpaau/chilen/archive/refs/heads/main.tar.gz

BuildRequires:  cargo
BuildRequires:  alsa-lib-devel
BuildRequires:  gcc

%description
%{summary}

Development snapshot of Chilen built from the latest commit on the main branch. Please don't
actually use this outside of testing.


%prep
%autosetup -n chilen-main


%build
cargo build --locked --release --features dev-opts --features mpris


%install
install -Dpm0755 target/release/chilen %{buildroot}%{_bindir}/%{name}

%files
%{_bindir}/%{name}
%license LICENSE
%doc README.md


%changelog
%autochangelog
