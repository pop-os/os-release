//! Type for parsing the `/etc/os-release` file.

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
    /// # Notes
    /// If an OS Release was not found, an error will be in its place.
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

fn parse_line(line: &str, skip: usize) -> &str {
    let line = line[skip..].trim();
    if line.starts_with('"') && line.ends_with('"') {
        &line[1..line.len() - 1]
    } else {
        line
    }
}

/// Contents of the `/etc/os-release` file, as a data structure.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsRelease {
    /// The URL where bugs should be reported for this OS.
    pub bug_report_url:     String,
    /// The homepage of this OS.
    pub home_url:           String,
    /// Identifier of the original upstream OS that this release is a derivative of.
    ///
    /// **IE:** `debian`
    pub id_like:            String,
    /// An identifier which describes this release, such as `ubuntu`.
    ///
    /// **IE:** `ubuntu`
    pub id:                 String,
    /// The name of this release, without the version string.
    ///
    /// **IE:** `Ubuntu`
    pub name:               String,
    /// The name of this release, with th eversion stirng.
    ///
    /// **IE:** `Ubuntu 18.04 LTS`
    pub pretty_name:        String,
    /// The URL describing this OS's privacy policy.
    pub privacy_policy_url: String,
    /// The URL for seeking support with this OS release.
    pub support_url:        String,
    /// The codename of this version.
    ///
    /// **IE:** `bionic`
    pub version_codename:   String,
    /// The version of this OS release, with additional details about the release.
    ///
    /// **IE:** `18.04 LTS (Bionic Beaver)`
    pub version_id:         String,
    /// The version of this OS release.
    ///
    /// **IE:** `18.04`
    pub version:            String,
    /// Additional keys not covered by the API.
    pub extra:              BTreeMap<String, String>
}

impl OsRelease {
    /// Attempt to parse the contents of `/etc/os-release`.
    pub fn new() -> io::Result<OsRelease> {
        let file = BufReader::new(open("/etc/os-release")?);
        Ok(OsRelease::from_iter(file.lines().flat_map(|line| line)))
    }

    /// Attempt to parse any `/etc/os-release`-like file.
    pub fn new_from<P: AsRef<Path>>(path: P) -> io::Result<OsRelease> {
        let file = BufReader::new(open(&path)?);
        Ok(OsRelease::from_iter(file.lines().flat_map(|line| line)))
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
                "PRETTY_NAME=" => os_release.pretty_name,
                "VERSION_ID=" => os_release.version_id,
                "HOME_URL=" => os_release.home_url,
                "SUPPORT_URL=" => os_release.support_url,
                "BUG_REPORT_URL=" => os_release.bug_report_url,
                "PRIVACY_POLICY_URL=" => os_release.privacy_policy_url,
                "VERSION_CODENAME=" => os_release.version_codename
            });

            if let Some(pos) = line.find('=') {
                if line.len() > pos+1 {
                    os_release.extra.insert(line[..pos].to_owned(), line[pos+1..].to_owned());
                }
            }
        }

        os_release
    }
}

fn open<P: AsRef<Path>>(path: P) -> io::Result<File> {
    File::open(&path).map_err(|why| io::Error::new(
        io::ErrorKind::Other,
        format!("unable to open file at {:?}: {}", path.as_ref(), why)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"NAME="Pop!_OS"
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
ANOTHER_KEY="#;

    #[test]
    fn os_release() {
        let os_release = OsRelease::from_iter(EXAMPLE.lines().map(|x| x.into()));

        assert_eq!(
            os_release,
            OsRelease {
                name:               "Pop!_OS".into(),
                version:            "18.04 LTS".into(),
                id:                 "ubuntu".into(),
                id_like:            "debian".into(),
                pretty_name:        "Pop!_OS 18.04 LTS".into(),
                version_id:         "18.04".into(),
                home_url:           "https://system76.com/pop".into(),
                support_url:        "http://support.system76.com".into(),
                bug_report_url:     "https://github.com/pop-os/pop/issues".into(),
                privacy_policy_url: "https://system76.com/privacy".into(),
                version_codename:   "bionic".into(),
                extra: {
                    let mut map = BTreeMap::new();
                    map.insert("EXTRA_KEY".to_owned(), "thing".to_owned());
                    map
                }
            }
        )
    }
}
