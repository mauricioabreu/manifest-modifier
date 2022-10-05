use m3u8_rs::{MasterPlaylist, MediaPlaylist, MediaSegment, Playlist};

#[derive(Debug)]
pub struct BandwidthFilter {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Debug)]
pub struct TrimFilter {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

pub fn load_master(content: &[u8]) -> Result<MasterPlaylist, String> {
    match m3u8_rs::parse_playlist(content) {
        Result::Ok((_, Playlist::MasterPlaylist(pl))) => Ok(pl),
        Result::Ok((_, Playlist::MediaPlaylist(_))) => Err("Must be a master playlist".to_string()),
        Result::Err(e) => Err(e.to_string()),
    }
}

pub fn filter_fps(pl: MasterPlaylist, rate: Option<f64>) -> MasterPlaylist {
    match rate {
        Some(r) => {
            let mut mpl = pl.clone();
            mpl.variants = pl
                .variants
                .into_iter()
                .filter(|v| v.frame_rate == Some(r))
                .collect::<Vec<m3u8_rs::VariantStream>>();
            mpl
        }
        None => pl,
    }
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

pub fn filter_dvr(pl: MediaPlaylist, seconds: Option<u64>) -> MediaPlaylist {
    let mut acc = 0;
    let mut mpl = pl.clone();

    let mut segments: Vec<MediaSegment> = Vec::new();
    match seconds {
        Some(s) => {
            for segment in pl.segments.iter().rev() {
                acc += segment.duration as u64;
                if acc <= s {
                    segments.push(segment.clone());
                    continue;
                }
                break;
            }
            mpl.segments = segments;
            mpl
        }
        None => pl,
    }
}

pub fn trim(pl: MediaPlaylist, opts: TrimFilter) -> MediaPlaylist {
    let start = opts.start.unwrap_or(0);
    let end = opts.end.unwrap_or(pl.segments.len().try_into().unwrap());

    let segments = &pl.segments[start as usize..end as usize];
    let mut mpl = pl.clone();
    mpl.segments = segments.to_vec();
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
        let nmp = filter_fps(master_playlist, Some(60.0));

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

    #[test]
    fn filter_dvr_with_short_duration() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let nmp = filter_dvr(media_playlist, Some(15));

        assert_eq!(nmp.segments.len(), 3);
    }

    #[test]
    fn filter_dvr_with_long_duration() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let nmp = filter_dvr(media_playlist.clone(), Some(u64::MAX));

        assert_eq!(nmp.segments.len(), media_playlist.segments.len());
    }

    #[test]
    fn trim_media_playlist_with_start_only() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let nmp = trim(
            media_playlist,
            TrimFilter {
                start: Some(5),
                end: None,
            },
        );

        assert_eq!(nmp.segments.len(), 15);
    }

    #[test]
    fn trim_media_playlist_with_end_only() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let nmp = trim(
            media_playlist,
            TrimFilter {
                start: None,
                end: Some(5),
            },
        );

        assert_eq!(nmp.segments.len(), 5);
    }

    #[test]
    fn trim_media_playlist_with_start_and_end() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let nmp = trim(
            media_playlist,
            TrimFilter {
                start: Some(5),
                end: Some(18),
            },
        );

        assert_eq!(nmp.segments.len(), 13);
    }
}
