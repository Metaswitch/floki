use std::path;
use std::{collections::BTreeMap, os::unix::prelude::OsStrExt};

use sha2::{Digest, Sha256};

use crate::config::Volume;

static VOLUME_DIRECTORY: &str = "volumes/";

pub(crate) fn resolve_volume_mounts<'a>(
    config_filepath: &path::Path,
    work_path: &path::Path,
    volumes: &'a BTreeMap<String, Volume>,
) -> Vec<(path::PathBuf, &'a path::PathBuf)> {
    volumes
        .iter()
        .map(|(name, volume)| {
            (
                cache_path(work_path, config_filepath, name, volume),
                &volume.mount,
            )
        })
        .collect()
}

fn cache_path(
    work_path: &path::Path,
    config_filepath: &path::Path,
    name: &str,
    config: &Volume,
) -> path::PathBuf {
    let folder = prefix_cache(config.shared, config_filepath) + name;
    work_path.join(VOLUME_DIRECTORY).join::<String>(folder)
}

fn prefix_cache(shared: bool, config_filepath: &path::Path) -> String {
    if shared {
        "".into()
    } else {
        hash_path(config_filepath) + "-"
    }
}

fn hash_path(path: &path::Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_os_str().as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_shared_cache_path_is_shared_across_flokis() {
        let cache_1 = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/1/floki.yaml"),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );
        let cache_2 = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/2/floki.yaml"),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );

        assert_eq!(cache_1, cache_2);
    }

    #[test]
    fn test_local_cache_path_is_not_shared_across_flokis() {
        let cache_1 = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/1/floki.yaml"),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );
        let cache_2 = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/2/floki.yaml"),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );

        assert_ne!(cache_1, cache_2);
    }

    #[test]
    fn test_local_and_shared_caches_dont_collide() {
        let cache_shared = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/1/floki.yaml"),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );
        let cache_local = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/1/floki.yaml"),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );

        assert_ne!(cache_shared, cache_local);
    }

    #[test]
    fn test_local_volumes_from_different_configs_dont_collide() {
        let cache_shared = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/1/floki-alternate.yaml"),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );
        let cache_local = cache_path(
            Path::new("work_path"),
            Path::new("/floki/root/1/floki.yaml"),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );

        assert_ne!(cache_shared, cache_local);
    }

    #[test]
    fn test_path_sha() {
        let path = Path::new("/floki/root/1/floki.yaml");
        let hash = hash_path(path);
        assert_eq!(
            hash,
            "04820cace8be1a2e8057c92231963c269cc0fd0fef01fd3fdf2deaffb62dc48d"
        );
    }
}
