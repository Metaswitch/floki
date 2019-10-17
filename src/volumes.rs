use std::collections::BTreeMap;
use std::path;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use crate::config::Volume;

pub(crate) fn resolve_volume_mounts(
    floki_root: &path::PathBuf,
    work_path: &path::PathBuf,
    volumes: &BTreeMap<String, Volume>,
) -> Vec<(path::PathBuf, path::PathBuf)> {
    volumes
        .iter()
        .map(|(name, volume)| {
            (
                cache_path(work_path, floki_root, name, volume),
                volume.mount.clone(),
            )
        })
        .collect()
}

fn cache_path(
    work_path: &path::PathBuf,
    floki_root: &path::PathBuf,
    name: &str,
    config: &Volume,
) -> path::PathBuf {
    let folder = prefix_cache(config.shared, floki_root) + name;
    work_path.join("cache/").join::<String>(folder)
}

fn prefix_cache(shared: bool, floki_root: &path::PathBuf) -> String {
    if shared {
        "".into()
    } else {
        hash_path(floki_root) + "-"
    }
}

fn hash_path(path: &path::PathBuf) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(&path.as_os_str().to_string_lossy());
    hasher.result_str()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shared_cache_path_is_shared_across_flokis() {
        let cache_1 = cache_path(
            &"work_path".into(),
            &"/floki/root/1/".into(),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );
        let cache_2 = cache_path(
            &"work_path".into(),
            &"/floki/root/2/".into(),
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
            &"work_path".into(),
            &"/floki/root/1/".into(),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );
        let cache_2 = cache_path(
            &"work_path".into(),
            &"/floki/root/2/".into(),
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
            &"work_path".into(),
            &"/floki/root/1/".into(),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );
        let cache_local = cache_path(
            &"work_path".into(),
            &"/floki/root/1/".into(),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );

        assert_ne!(cache_shared, cache_local);
    }
}
