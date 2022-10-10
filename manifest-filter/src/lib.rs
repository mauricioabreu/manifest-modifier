//! manifest-filter is a lib used to modify video manifests.
//!
//! # Table of contents
//!
//! - [Features](#features)
//! - [Examples](#examples)
//!
//! # Features
//!
//! - Modify master playlists (filter variants by bandwidth, fps, etc)
//! - Modify media playlists (DVR, trim segments)
//!
//! More features are coming soon.
//!
//! # Examples
//!
//! Below you can try the example used to filter only the variants that are 30fps.
//!
//! ```rust
//! use manifest_filter::Master;
//! use std::io::Read;
//!
//! let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
//! let mut content: Vec<u8> = Vec::new();
//! file.read_to_end(&mut content).unwrap();
//!
//! let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
//! let mut master = Master {
//!     playlist: master_playlist,
//! };
//! master.filter_fps(Some(30.0));
//! ```
//!
//! The result should be something like this
//!
//! ```not_rust
//! #EXTM3U
//! #EXT-X-VERSION:4
//! #EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio-aach-96",LANGUAGE="en",NAME="English",DEFAULT=YES,AUTOSELECT=YES,CHANNELS="2"
//! #EXT-X-STREAM-INF:BANDWIDTH=600000,AVERAGE-BANDWIDTH=600000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=384x216,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
//! variant-audio_1=96000-video=249984.m3u8
//! #EXT-X-STREAM-INF:BANDWIDTH=800000,AVERAGE-BANDWIDTH=800000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=768x432,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
//! variant-audio_1=96000-video=1320960.m3u8
//! ```
//!
//! All functions can be chained. Call `filter_fps` to first remove variants with
//! a frame rate different of the one choosen and call `filter_bandwith` right after to
//! remove variants thare don't fit into the max/min range expected.
//! The sampe applies for `Media`.

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
        Result::Ok((_, Playlist::MediaPlaylist(_))) => Err("must be a master playlist".to_string()),
        Result::Err(e) => Err(e.to_string()),
    }
}

pub fn load_media(content: &[u8]) -> Result<MediaPlaylist, String> {
    match m3u8_rs::parse_playlist(content) {
        Result::Ok((_, Playlist::MediaPlaylist(pl))) => Ok(pl),
        Result::Ok((_, Playlist::MasterPlaylist(_))) => Err("must be a media playlist".to_string()),
        Result::Err(e) => Err(e.to_string()),
    }
}

/// `Master` holds a reference to the master playlist. All
/// functions implemented by this struct can be chained.
pub struct Master {
    pub playlist: MasterPlaylist,
}

/// `Media` holds a reference to the media playlist. All
/// functions implemented by this struct can be chained.
pub struct Media {
    pub playlist: MediaPlaylist,
}

impl Master {
    /// Filter variants from a master playlist based on the frame rate passed.
    pub fn filter_fps(&mut self, rate: Option<f64>) -> &mut Self {
        if let Some(r) = rate {
            self.playlist.variants.retain(|v| v.frame_rate == Some(r));
        }
        self
    }

    /// Filter variants from a master playlist based on the bandwidh passed.
    ///
    /// Variants can be filtered using `min` and `max` values for bandwidth.
    ///
    /// There's no need to pass a `min` value if you don't need to. The
    /// same happens for `max` value. For `min` we will set to zero by default
    /// and for the `max` we'll use the `u64::MAX` value.
    pub fn filter_bandwidth(&mut self, opts: BandwidthFilter) -> &mut Self {
        let min = opts.min.unwrap_or(0);
        let max = opts.max.unwrap_or(u64::MAX);

        self.playlist
            .variants
            .retain(|v| v.bandwidth >= min && v.bandwidth <= max);
        self
    }

    /// Set the first variant to appear in the playlist for the one that
    /// best suites the device needs. Most of the times such feature will
    /// be used to skip the initial variant (too low for some devices).
    ///
    /// # Arguments
    /// * `index` - an Option containing the index you want to be the first variant. Variants will be swapped.
    /// * `closest_bandwidth` - an Option containing the closed bandwidth value you want for the first variant.
    pub fn first_variant(
        &mut self,
        index: Option<u64>,
        closest_bandwidth: Option<u64>,
    ) -> &mut Self {
        if let Some(i) = index {
            self.playlist.variants.swap(0, i.try_into().unwrap());
            self
        } else if let Some(c) = closest_bandwidth {
            let (idx, _) = self.playlist.variants.iter().enumerate().min_by_key(|(_, v)| (c as i64 - v.bandwidth as i64).abs()).unwrap();
            let fv = self.playlist.variants.remove(idx);
            self.playlist.variants.insert(0, fv);
            self
        } else {
            self
        }
    }
}

impl Media {
    /// Remove segments backwards from the media playlist, based on the duration
    /// set. The duration is in seconds.
    /// Media sequence will be affected: https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.3.2
    pub fn filter_dvr(&mut self, seconds: Option<u64>) -> &mut Self {
        let mut acc = 0;
        let total_segments = self.playlist.segments.len();

        if let Some(s) = seconds {
            self.playlist.segments = self
                .playlist
                .segments
                .iter()
                .rev()
                .take_while(|segment| {
                    acc += segment.duration as u64;
                    acc <= s
                })
                .cloned()
                .collect();
            self.playlist.media_sequence += (total_segments - self.playlist.segments.len()) as u64;
        }
        self
    }

    /// Remove segments from the media playlist, based on the start/end passed.
    /// Media sequence will be affected: https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.3.2
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
        master.filter_fps(Some(60.0));

        assert_eq!(master.playlist.variants.len(), 2);
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
        master.filter_bandwidth(BandwidthFilter {
            min: Some(800000),
            max: None,
        });

        assert_eq!(master.playlist.variants.len(), 3);
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
        master.filter_bandwidth(BandwidthFilter {
            min: None,
            max: Some(800000),
        });

        assert_eq!(master.playlist.variants.len(), 6);
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
        master.filter_bandwidth(BandwidthFilter {
            min: Some(800000),
            max: Some(2000000),
        });

        assert_eq!(master.playlist.variants.len(), 3);
    }

    #[test]
    fn set_first_variant_by_index() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let mut master = Master {
            playlist: master_playlist,
        };
        master.first_variant(Some(1), None);

        assert_eq!(master.playlist.variants[0].bandwidth, 800000);
        assert_eq!(master.playlist.variants[1].bandwidth, 600000);
    }

    #[test]
    fn set_first_variant_by_closest_bandwidth() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
        let mut master = Master {
            playlist: master_playlist,
        };
        master.first_variant(None, Some(1650000));
        assert_eq!(master.playlist.variants[0].bandwidth, 1500000);
        assert_eq!(master.playlist.variants[1].bandwidth, 600000);
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
        media.filter_dvr(Some(15));

        assert_eq!(media.playlist.segments.len(), 3);
        assert_eq!(media.playlist.media_sequence, 320035373);
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
        media.filter_dvr(Some(u64::MAX));

        assert_eq!(media.playlist.segments.len(), 20);
        assert_eq!(media.playlist.media_sequence, 320035356);
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
        media.trim(TrimFilter {
            start: Some(5),
            end: None,
        });

        assert_eq!(media.playlist.segments.len(), 15);
        assert_eq!(media.playlist.media_sequence, 320035361);
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
        media.trim(TrimFilter {
            start: None,
            end: Some(5),
        });

        assert_eq!(media.playlist.segments.len(), 5);
        assert_eq!(media.playlist.media_sequence, 320035371);
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
        media.trim(TrimFilter {
            start: Some(5),
            end: Some(18),
        });

        assert_eq!(media.playlist.segments.len(), 13);
        assert_eq!(media.playlist.media_sequence, 320035363);
    }
}
