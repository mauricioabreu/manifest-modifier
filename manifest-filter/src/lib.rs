use m3u8_rs::MasterPlaylist;

#[derive(Debug)]
pub struct BandwidthFilter {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

pub fn filter_fps(pl: MasterPlaylist, rate: f64) -> MasterPlaylist {
    let mut mpl = pl.clone();
    mpl.variants = pl
        .variants
        .into_iter()
        .filter(|v| v.frame_rate == Some(rate))
        .collect::<Vec<m3u8_rs::VariantStream>>();
    mpl
}

pub fn filter_bandwidth(pl: MasterPlaylist, opts: BandwidthFilter) -> MasterPlaylist {
    let min = opts.min.unwrap_or(0);
    let max = opts.max.unwrap_or(u64::MAX);

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

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let nmp = filter_fps(master_playlist, 60.0);

        assert_eq!(nmp.variants.len(), 2);
    }

    #[test]
    fn filter_min_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let nmp = filter_bandwidth(
            master_playlist,
            BandwidthFilter {
                min: Some(800000),
                max: None,
            },
        );

        assert_eq!(nmp.variants.len(), 3);
    }

    #[test]
    fn filter_max_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let nmp = filter_bandwidth(
            master_playlist,
            BandwidthFilter {
                min: None,
                max: Some(800000),
            },
        );

        assert_eq!(nmp.variants.len(), 6);
    }

    #[test]
    fn filter_min_and_max_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let nmp = filter_bandwidth(
            master_playlist,
            BandwidthFilter {
                min: Some(800000),
                max: Some(2000000),
            },
        );

        assert_eq!(nmp.variants.len(), 3);
    }
}
