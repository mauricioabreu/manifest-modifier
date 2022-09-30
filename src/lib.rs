use m3u8_rs::MasterPlaylist;

#[derive(Debug)]
struct BandwidthFilter {
    Min: Option<u64>,
    Max: Option<u64>,
}

fn filter_fps(pl: MasterPlaylist, rate: f64) -> MasterPlaylist {
    let mut mpl = pl.clone();
    mpl.variants = pl
        .variants
        .into_iter()
        .filter(|v| v.frame_rate == Some(rate))
        .collect::<Vec<m3u8_rs::VariantStream>>();
    mpl
}

fn filter_bandwidth(pl: MasterPlaylist, opts: BandwidthFilter) -> MasterPlaylist {
    let min = opts.Min.unwrap_or(0);
    let max = opts.Max.unwrap_or(u64::MAX);

    let mut mpl = pl.clone();
    mpl.variants = pl
        .variants
        .into_iter()
        .filter(|v| v.bandwidth >= min && v.bandwidth <= max)
        .collect::<Vec<m3u8_rs::VariantStream>>();
    mpl
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn filter_60_fps() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        if let Result::Ok((_, master_playlist)) = m3u8_rs::parse_master_playlist(&content) {
            let nmp = filter_fps(master_playlist, 60.0);
            assert_eq!(nmp.variants.len(), 2);
        };
    }

    #[test]
    fn filter_min_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        if let Result::Ok((_, master_playlist)) = m3u8_rs::parse_master_playlist(&content) {
            let nmp = filter_bandwidth(
                master_playlist,
                BandwidthFilter {
                    Min: Some(800000),
                    Max: None,
                },
            );
            assert_eq!(nmp.variants.len(), 3);
        }
    }
}
