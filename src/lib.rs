//! Type for parsing the `/etc/os-release` file.
//!
//! For the semantics of this file, see
//! [https://www.freedesktop.org/software/systemd/man/os-release.html](https://www.freedesktop.org/software/systemd/man/os-release.html).

#[macro_use]
extern crate lazy_static;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::iter::FromIterator;
use std::path::Path;

lazy_static! {
    /// The OS release detected on this host's environment.
    ///
    /// This is created with `OsRelease::new()` which uses `/etc/os-release`, if available, or
    /// `/usr/lib/os-release` if not.
    ///
    /// # Notes
    /// If an OS Release file was not found, an error will be in its place.
    pub static ref OS_RELEASE: io::Result<OsRelease> = OsRelease::new();
}

macro_rules! map_keys {
    ($item:expr, { $($pat:expr => $field:expr),+ }) => {{
        $(
            if $item.starts_with($pat) {
                $field = parse_line($item, $pat.len()).into();
                continue;
            }
        )+
    }};
}

fn is_enclosed_with(line: &str, pattern: char) -> bool {
    line.starts_with(pattern) && line.ends_with(pattern)
}

fn parse_line(line: &str, skip: usize) -> &str {
    let line = line[skip..].trim();
    if is_enclosed_with(line, '"') || is_enclosed_with(line, '\'') {
        &line[1..line.len() - 1]
    } else {
        line
    }
}

/// Contents of the `/etc/os-release` file, as a data structure.
///
/// See
/// [https://www.freedesktop.org/software/systemd/man/os-release.html](https://www.freedesktop.org/software/systemd/man/os-release.html)
/// for further documentation on the fields and semantics.
///
/// Quotes are removed from strings however escape sequences are not parsed.
///
/// Optional fialds which are not present default to `""`.
#[derive(Clone, Debug, PartialEq)]
pub struct OsRelease {
    /// The name of this release, without the version string.
    ///
    /// Defaults to `Linux`.
    ///
    /// **IE:** `Ubuntu`
    pub name: String,

    /// The version of this OS release, excluding the OS name.
    ///
    /// This field is optional.
    ///
    /// **IE:** `18.04 LTS (Bionic Beaver)`
    pub version: String,

    /// An identifier which describes the OS, excluding the version, such as `ubuntu`.
    /// This should be a string consisting only of 0-9, a-z, ".", "_", "-" (no upper case letters).
    ///
    /// Defaults to `linux`.
    ///
    /// **IE:** `ubuntu`
    pub id: String,

    /// A space separated list of identifiers for operating systems which are closely related to
    /// this OS (likely the operating systems it is derived from).
    /// Each item should be a string consisting only of 0-9, a-z, ".", "_", "-" (no upper case
    /// letters).
    ///
    /// This field is optional.
    ///
    /// **IE:** `debian`
    pub id_like: String,

    /// The codename of this version.
    /// This should be a string consisting only of 0-9, a-z, ".", "_", "-" (no upper case letters).
    ///
    /// This field is optional.
    ///
    /// **IE:** `bionic`
    pub version_codename: String,

    /// Identifier for the version of this OS.
    /// This should be a string consisting only of 0-9, a-z, ".", "_", "-" (no upper case letters).
    ///
    /// This field is optional.
    ///
    /// **IE:** `18.04`
    pub version_id: String,

    /// A pretty name for this OS which can be presented to the user.
    ///
    /// Defaults to `Linux`.
    ///
    /// **IE:** `Pop!_OS 18.04 LTS`
    pub pretty_name: String,

    /// The suggested presentation color when showing the OS name in the console.
    /// Given in a format suitable for inclusion in the ESC [ m ANSI/ECMA-48 escape code.
    ///
    /// This field is optional.
    /// Since this field is optional, it may be "" (don't forget to check before printing).
    ///
    /// **IE:** `1;31` (red) or `38;2;23;147;209` (RGB 23, 147, 209)
    pub ansi_color: String,

    /// A CPE name for the operating system, in URI binding syntax.
    /// See the Common Platform Enumeration Specification.
    ///
    /// This field is optional.
    ///
    /// **EG:** `cpe:/o:fedoraproject:fedora:17`
    pub cpe_name: String,

    /// The homepage of this OS.
    ///
    /// This field is optional.
    pub home_url: String,
    /// The documentation page of this OS.
    ///
    /// This field is optional.
    pub documentation_url: String,
    /// The URL for seeking support with this OS release.
    ///
    /// This field is optional.
    pub support_url: String,
    /// The URL where bugs should be reported for this OS.
    ///
    /// This field is optional.
    pub bug_report_url: String,
    /// The URL describing this OS's privacy policy.
    ///
    /// This field is optional.
    pub privacy_policy_url: String,

    /// A unique ID for the image used as the origin for the OS (it is not updated).
    ///
    /// This field is optional.
    pub build_id: String,

    /// The variant or edition of the OS.
    ///
    /// This field is optional.
    ///
    /// **EG:** `Server`
    pub variant: String,

    /// An ID for the variant or edition of the OS.
    /// This should be a string consisting only of 0-9, a-z, ".", "_", "-" (no upper case letters).
    ///
    /// This field is optional.
    pub variant_id: String,

    /// The name of the logo for the operating system, as defined by freedesktop.org Icon Theme
    /// Specification.
    ///
    /// This field is optional.
    pub logo: String,

    /// Additional keys not covered by the API.
    pub extra: BTreeMap<String, String>,
}

impl Default for OsRelease {
    fn default() -> OsRelease {
        OsRelease {
            name: "Linux".into(),
            id: "linux".into(),
            pretty_name: "Linux".into(),

            version: String::new(),
            id_like: String::new(),
            version_codename: String::new(),
            version_id: String::new(),
            ansi_color: String::new(),
            cpe_name: String::new(),
            home_url: String::new(),
            documentation_url: String::new(),
            support_url: String::new(),
            bug_report_url: String::new(),
            privacy_policy_url: String::new(),
            build_id: String::new(),
            variant: String::new(),
            variant_id: String::new(),
            logo: String::new(),

            extra: BTreeMap::default(),
        }
    }
}

impl OsRelease {
    /// Attempt to parse the contents of `/etc/os-release`.
    /// Falls back to `/usr/lib/os-release`.
    pub fn new() -> io::Result<OsRelease> {
        let file = BufReader::new(open("/etc/os-release").or_else(|first_err| {
            open("/usr/lib/os-release").map_err(|second_err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("{} then {}", first_err, second_err),
                )
            })
        })?);
        Ok(OsRelease::from_iter(file.lines().flatten()))
    }

    /// Attempt to parse any `/etc/os-release`-like file.
    pub fn new_from<P: AsRef<Path>>(path: P) -> io::Result<OsRelease> {
        let file = BufReader::new(open(&path)?);
        Ok(OsRelease::from_iter(file.lines().flatten()))
    }
}

impl FromIterator<String> for OsRelease {
    fn from_iter<I: IntoIterator<Item = String>>(lines: I) -> Self {
        let mut os_release = Self::default();

        for line in lines {
            let line = line.trim();

            map_keys!(line, {
                "NAME=" => os_release.name,
                "VERSION=" => os_release.version,
                "ID=" => os_release.id,
                "ID_LIKE=" => os_release.id_like,
                "VERSION_ID=" => os_release.version_id,
                "VERSION_CODENAME=" => os_release.version_codename,
                "PRETTY_NAME=" => os_release.pretty_name,
                "ANSI_COLOR=" => os_release.ansi_color,
                "CPE_NAME=" => os_release.cpe_name,
                "HOME_URL=" => os_release.home_url,
                "DOCUMENTATION_URL=" => os_release.documentation_url,
                "SUPPORT_URL=" => os_release.support_url,
                "BUG_REPORT_URL=" => os_release.bug_report_url,
                "PRIVACY_POLICY_URL=" => os_release.privacy_policy_url,
                "BUILD_ID=" => os_release.build_id,
                "VARIANT=" => os_release.variant,
                "VARIANT_ID=" => os_release.variant_id,
                "LOGO=" => os_release.logo
            });

            if let Some(pos) = line.find('=') {
                if line.len() > pos + 1 {
                    os_release
                        .extra
                        .insert(line[..pos].to_owned(), line[pos + 1..].to_owned());
                }
            }
        }

        os_release
    }
}

fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::open(&path).map_err(|why| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("unable to open file at {:?}: {}", path.as_ref(), why),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const POP: &str = r#"NAME="Pop!_OS"
VERSION="18.04 LTS"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Pop!_OS 18.04 LTS"
VERSION_ID="18.04"
HOME_URL="https://system76.com/pop"
SUPPORT_URL="http://support.system76.com"
BUG_REPORT_URL="https://github.com/pop-os/pop/issues"
PRIVACY_POLICY_URL="https://system76.com/privacy"
VERSION_CODENAME=bionic
EXTRA_KEY=thing
ANOTHER_KEY=
YET_ANOTHER_KEY=5"#;
    #[test]
    fn os_release_pop() {
        let os_release = OsRelease::from_iter(POP.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name: "Pop!_OS".into(),
                version: "18.04 LTS".into(),
                id: "ubuntu".into(),
                id_like: "debian".into(),
                pretty_name: "Pop!_OS 18.04 LTS".into(),
                version_id: "18.04".into(),
                home_url: "https://system76.com/pop".into(),
                support_url: "http://support.system76.com".into(),
                bug_report_url: "https://github.com/pop-os/pop/issues".into(),
                privacy_policy_url: "https://system76.com/privacy".into(),
                version_codename: "bionic".into(),
                extra: {
                    let mut map = BTreeMap::new();
                    map.insert("EXTRA_KEY".to_owned(), "thing".to_owned());
                    map.insert("YET_ANOTHER_KEY".to_owned(), "5".to_owned());
                    map
                },
                ..OsRelease::default()
            }
        )
    }
    const FEDORA: &str = r#"
NAME=Fedora
VERSION="17 (Beefy Miracle)"
ID=fedora
VERSION_ID=17
PRETTY_NAME="Fedora 17 (Beefy Miracle)"
ANSI_COLOR="0;34"
CPE_NAME="cpe:/o:fedoraproject:fedora:17"
HOME_URL="https://fedoraproject.org/"
BUG_REPORT_URL="https://bugzilla.redhat.com/""#;
    #[test]
    fn os_release_fedora() {
        let os_release = OsRelease::from_iter(FEDORA.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name: "Fedora".into(),
                version: "17 (Beefy Miracle)".into(),
                id: "fedora".into(),
                pretty_name: "Fedora 17 (Beefy Miracle)".into(),
                ansi_color: "0;34".into(),
                version_id: "17".into(),
                cpe_name: "cpe:/o:fedoraproject:fedora:17".into(),
                home_url: "https://fedoraproject.org/".into(),
                bug_report_url: "https://bugzilla.redhat.com/".into(),
                extra: BTreeMap::new(),
                ..OsRelease::default()
            }
        )
    }
    const ARCH: &str = r#"NAME="Arch Linux"
PRETTY_NAME="Arch Linux"
ID=arch
# Comment
BUILD_ID=rolling
ANSI_COLOR="38;2;23;147;209"

# Comment
HOME_URL="https://www.archlinux.org/"
# Comment

DOCUMENTATION_URL="https://wiki.archlinux.org/"
SUPPORT_URL="https://bbs.archlinux.org/"

# Comment
#Another comment


BUG_REPORT_URL="https://bugs.archlinux.org/"
LOGO=archlinux

"#;
    #[test]
    fn os_release_arch() {
        let os_release = OsRelease::from_iter(ARCH.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name: "Arch Linux".into(),
                pretty_name: "Arch Linux".into(),
                build_id: "rolling".into(),
                id: "arch".into(),
                ansi_color: "38;2;23;147;209".into(),
                home_url: "https://www.archlinux.org/".into(),
                documentation_url: "https://wiki.archlinux.org/".into(),
                support_url: "https://bbs.archlinux.org/".into(),
                bug_report_url: "https://bugs.archlinux.org/".into(),
                logo: "archlinux".into(),
                extra: BTreeMap::new(),
                ..OsRelease::default()
            }
        )
    }
    const UBUNTU: &str = r#"NAME="Ubuntu"
VERSION="18.04.4 LTS (Bionic Beaver)"
ID=ubuntu
ID_LIKE=debian

PRETTY_NAME="Ubuntu 18.04.4 LTS"
VERSION_ID="18.04"


HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"



VERSION_CODENAME=bionic
UBUNTU_CODENAME=bionic
"#;
    #[test]
    fn os_release_ubuntu() {
        let os_release = OsRelease::from_iter(UBUNTU.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name: "Ubuntu".into(),
                version: "18.04.4 LTS (Bionic Beaver)".into(),
                id: "ubuntu".into(),
                id_like: "debian".into(),
                pretty_name: "Ubuntu 18.04.4 LTS".into(),
                version_id: "18.04".into(),
                home_url: "https://www.ubuntu.com/".into(),
                support_url: "https://help.ubuntu.com/".into(),
                bug_report_url: "https://bugs.launchpad.net/ubuntu/".into(),
                privacy_policy_url:
                    "https://www.ubuntu.com/legal/terms-and-policies/privacy-policy".into(),
                version_codename: "bionic".into(),
                extra: {
                    let mut map = BTreeMap::new();
                    map.insert("UBUNTU_CODENAME".to_owned(), "bionic".to_owned());
                    map
                },
                ..OsRelease::default()
            }
        )
    }
    const NOTHING: &str = "

        ";
    #[test]
    fn os_release_nothing() {
        let os_release = OsRelease::from_iter(NOTHING.lines().map(|x| x.into()));

        assert_eq!(os_release, OsRelease::default());
        assert_eq!(
            os_release,
            OsRelease {
                name: "Linux".into(),
                pretty_name: "Linux".into(),
                id: "linux".into(),
                ..OsRelease::default()
            }
        )
    }
    const JUST_EXTRA: &str = "EXTRA=test";
    #[test]
    fn os_release_just_extra() {
        let os_release = OsRelease::from_iter(JUST_EXTRA.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                extra: {
                    let mut map = BTreeMap::new();
                    map.insert("EXTRA".to_owned(), "test".to_owned());
                    map
                },
                ..OsRelease::default()
            }
        )
    }
    const ALL: &str = r#"NAME=1
VERSION=2
ID=3
ID_LIKE=4
PRETTY_NAME=5
ANSI_COLOR=6
VERSION_ID=7
HOME_URL=8
SUPPORT_URL=9
BUG_REPORT_URL=a
PRIVACY_POLICY_URL=B
VERSION_CODENAME=C
CPE_NAME=D
DOCUMENTATION_URL=E
BUILD_ID=F
VARIANT=G
VARIANT_ID=H
LOGO=I"#;
    #[test]
    fn os_release_all() {
        let os_release = OsRelease::from_iter(ALL.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name: "1".into(),
                version: "2".into(),
                id: "3".into(),
                id_like: "4".into(),
                pretty_name: "5".into(),
                ansi_color: "6".into(),
                version_id: "7".into(),
                home_url: "8".into(),
                support_url: "9".into(),
                bug_report_url: "a".into(),
                privacy_policy_url: "B".into(),
                version_codename: "C".into(),
                cpe_name: "D".into(),
                documentation_url: "E".into(),
                build_id: "F".into(),
                variant: "G".into(),
                variant_id: "H".into(),
                logo: "I".into(),
                extra: BTreeMap::new(),
            }
        )
    }
}
