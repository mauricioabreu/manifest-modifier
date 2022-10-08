use m3u8_rs::{MasterPlaylist, MediaPlaylist, Playlist};

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

pub fn load_media(content: &[u8]) -> Result<MediaPlaylist, String> {
    match m3u8_rs::parse_playlist(content) {
        Result::Ok((_, Playlist::MediaPlaylist(pl))) => Ok(pl),
        Result::Ok((_, Playlist::MasterPlaylist(_))) => Err("Must be a media playlist".to_string()),
        Result::Err(e) => Err(e.to_string()),
    }
}

pub struct Master {
    pub playlist: MasterPlaylist,
}

pub struct Media {
    pub playlist: MediaPlaylist,
}

impl Master {
    pub fn filter_fps(&mut self, rate: Option<f64>) -> &mut Self {
        if let Some(r) = rate {
            self.playlist.variants.retain(|v| v.frame_rate == Some(r));
        }
        self
    }

    pub fn filter_bandwidth(&mut self, opts: BandwidthFilter) -> &mut Self {
        let min = opts.min.unwrap_or(0);
        let max = opts.max.unwrap_or(u64::MAX);

        self.playlist.variants.retain(|v| v.bandwidth >= min && v.bandwidth <= max);
        self
    }
}

impl Media {
    pub fn filter_dvr(&mut self, seconds: Option<u64>) -> &mut Self {
        let mut acc = 0;
        let total_segments = self.playlist.segments.len();

        match seconds {
            Some(s) => {
                self.playlist.segments = self
                    .playlist
                    .clone()
                    .segments
                    .iter()
                    .rev()
                    .take_while(|segment| {
                        acc += segment.duration as u64;
                        acc <= s
                    })
                    .cloned()
                    .collect();
                self.playlist.media_sequence +=
                    (total_segments - self.playlist.segments.len()) as u64;
                self
            }
            None => self,
        }
    }

    pub fn trim(&mut self, opts: TrimFilter) -> &mut Self {
        let start = opts.start.unwrap_or(0);
        let end = opts
            .end
            .unwrap_or_else(|| self.playlist.segments.len().try_into().unwrap());

        let segments = &self.playlist.segments[start as usize..end as usize];
        let total_segments = self.playlist.segments.len();
        self.playlist.segments = segments.to_vec();
        self.playlist.media_sequence += (total_segments - self.playlist.segments.len()) as u64;
        self
    }
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
        let mut master = Master {
            playlist: master_playlist,
        };
        let nmp = master.filter_fps(Some(60.0));

        assert_eq!(nmp.playlist.variants.len(), 2);
    }

    #[test]
    fn filter_min_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let mut master = Master {
            playlist: master_playlist,
        };
        let nmp = master.filter_bandwidth(BandwidthFilter {
            min: Some(800000),
            max: None,
        });

        assert_eq!(nmp.playlist.variants.len(), 3);
    }

    #[test]
    fn filter_max_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let mut master = Master {
            playlist: master_playlist,
        };
        let nmp = master.filter_bandwidth(BandwidthFilter {
            min: None,
            max: Some(800000),
        });

        assert_eq!(nmp.playlist.variants.len(), 6);
    }

    #[test]
    fn filter_min_and_max_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let mut master = Master {
            playlist: master_playlist,
        };
        let nmp = master.filter_bandwidth(BandwidthFilter {
            min: Some(800000),
            max: Some(2000000),
        });

        assert_eq!(nmp.playlist.variants.len(), 3);
    }

    #[test]
    fn filter_dvr_with_short_duration() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let mut media = Media {
            playlist: media_playlist,
        };
        let nmp = media.filter_dvr(Some(15));

        assert_eq!(nmp.playlist.segments.len(), 3);
        assert_eq!(nmp.playlist.media_sequence, 320035373);
    }

    #[test]
    fn filter_dvr_with_long_duration() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let mut media = Media {
            playlist: media_playlist,
        };
        let nmp = media.filter_dvr(Some(u64::MAX));

        assert_eq!(nmp.playlist.segments.len(), 20);
        assert_eq!(nmp.playlist.media_sequence, 320035356);
    }

    #[test]
    fn trim_media_playlist_with_start_only() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let mut media = Media {
            playlist: media_playlist,
        };
        let nmp = media.trim(TrimFilter {
            start: Some(5),
            end: None,
        });

        assert_eq!(nmp.playlist.segments.len(), 15);
        assert_eq!(nmp.playlist.media_sequence, 320035361);
    }

    #[test]
    fn trim_media_playlist_with_end_only() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let mut media = Media {
            playlist: media_playlist,
        };
        let nmp = media.trim(TrimFilter {
            start: None,
            end: Some(5),
        });

        assert_eq!(nmp.playlist.segments.len(), 5);
        assert_eq!(nmp.playlist.media_sequence, 320035371);
    }

    #[test]
    fn trim_media_playlist_with_start_and_end() {
        let mut file = std::fs::File::open("manifests/media.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, media_playlist) = m3u8_rs::parse_media_playlist(&content).unwrap();
        let mut media = Media {
            playlist: media_playlist,
        };
        let nmp = media.trim(TrimFilter {
            start: Some(5),
            end: Some(18),
        });

        assert_eq!(nmp.playlist.segments.len(), 13);
        assert_eq!(nmp.playlist.media_sequence, 320035363);
    }
}
