/// Compares two version strings to determine their order, taking into account numerical segments,
/// pre-release identifiers, and build metadata.
///
/// The version comparison is performed as follows:
/// 1. The main version numbers are split by dots ('.') and compared numerically, segment by segment.
/// 2. If the main version numbers are equal, any pre-release identifiers (following a hyphen '-') are
///    compared. Numeric identifiers are compared numerically, while alphanumeric identifiers are compared
///    lexicographically.
/// 3. If the main version numbers and pre-release identifiers are equal, build metadata (following a '+')
///    is compared lexicographically.
///
/// # Arguments
///
/// * `version1` - The first version string to be compared.
/// * `version2` - The second version string to be compared.
///
/// # Returns
///
/// Returns `true` if `version1` is considered lower than `version2` according to the rules described above.
/// Returns `false` if `version1` is considered equal to or higher than `version2`.
pub(crate) fn compare_versions(version1: &str, version2: &str) -> bool {
    let parse_segment = |seg: &str| -> Vec<(String, Option<u32>)> {
        seg.split('.')
            .map(|s| {
                if let Ok(num) = s.parse::<u32>() {
                    (s.to_string(), Some(num))
                } else {
                    (s.to_string(), None)
                }
            })
            .collect()
    };

    let parse_version = |version: &str| -> (
        Vec<u32>,
        Vec<(String, Option<u32>)>,
        Vec<(String, Option<u32>)>,
    ) {
        let (main_pre, build_metadata) = version.split_once('+').unwrap_or((version, ""));
        let (main, pre_release) = main_pre.split_once('-').unwrap_or((main_pre, ""));
        let main_segments: Vec<u32> = main.split('.').filter_map(|s| s.parse().ok()).collect();
        let pre_release_segments = parse_segment(pre_release);
        let build_metadata_segments = parse_segment(build_metadata);
        (main_segments, pre_release_segments, build_metadata_segments)
    };

    let (v1_main, v1_pre, v1_build) = parse_version(version1);
    let (v2_main, v2_pre, v2_build) = parse_version(version2);

    for (seg1, seg2) in v1_main.iter().zip(v2_main.iter()) {
        if seg1 != seg2 {
            return seg1 < seg2;
        }
    }
    if v1_main.len() != v2_main.len() {
        return v1_main.len() < v2_main.len();
    }

    for (seg1, seg2) in v1_pre.iter().zip(v2_pre.iter()) {
        match (seg1.1, seg2.1) {
            (Some(num1), Some(num2)) => {
                if num1 != num2 {
                    return num1 < num2;
                }
            }
            (None, None) => {
                if seg1.0 != seg2.0 {
                    return seg1.0 < seg2.0;
                }
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
        }
    }

    if v1_pre.len() != v2_pre.len() {
        return v1_pre.len() < v2_pre.len();
    }

    for (seg1, seg2) in v1_build.iter().zip(v2_build.iter()) {
        match (seg1.1, seg2.1) {
            (Some(num1), Some(num2)) => {
                if num1 != num2 {
                    return num1 < num2;
                }
            }
            (None, None) => {
                if seg1.0 != seg2.0 {
                    return seg1.0 < seg2.0;
                }
            }
            (Some(_), None) => return true,
            (None, Some(_)) => return false,
        }
    }

    v1_build.len() < v2_build.len()
}
