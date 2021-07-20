use std::collections::BTreeMap;
use std::path;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use crate::config::Volume;

static VOLUME_DIRECTORY: &str = "volumes/";

pub(crate) fn resolve_volume_mounts<'a>(
    config_filepath: &path::PathBuf,
    work_path: &path::PathBuf,
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

pub(crate) fn cache_path(
    work_path: &path::PathBuf,
    config_filepath: &path::PathBuf,
    name: &str,
    config: &Volume,
) -> path::PathBuf {
    let folder = prefix_cache(config.shared, config_filepath) + name;
    work_path.join(VOLUME_DIRECTORY).join::<String>(folder)
}

fn prefix_cache(shared: bool, config_filepath: &path::PathBuf) -> String {
    if shared {
        "".into()
    } else {
        hash_path(config_filepath) + "-"
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
            &"/floki/root/1/floki.yaml".into(),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );
        let cache_2 = cache_path(
            &"work_path".into(),
            &"/floki/root/2/floki.yaml".into(),
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
            &"/floki/root/1/floki.yaml".into(),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );
        let cache_2 = cache_path(
            &"work_path".into(),
            &"/floki/root/2/floki.yaml".into(),
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
            &"/floki/root/1/floki.yaml".into(),
            "cache",
            &Volume {
                shared: true,
                mount: "/".into(),
            },
        );
        let cache_local = cache_path(
            &"work_path".into(),
            &"/floki/root/1/floki.yaml".into(),
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
            &"work_path".into(),
            &"/floki/root/1/floki-alternate.yaml".into(),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );
        let cache_local = cache_path(
            &"work_path".into(),
            &"/floki/root/1/floki.yaml".into(),
            "cache",
            &Volume {
                shared: false,
                mount: "/".into(),
            },
        );

        assert_ne!(cache_shared, cache_local);
    }
}
