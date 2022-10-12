[![test](https://github.com/mauricioabreu/manifest-modifier/actions/workflows/CI.yml/badge.svg)](https://github.com/mauricioabreu/manifest-modifier/actions/workflows/CI.yml)
[![Crates.io](https://img.shields.io/crates/v/manifest-filter)](https://crates.io/crates/manifest-filter)
[![manifest-filter docs](https://docs.rs/manifest-filter/badge.svg)](https://docs.rs/manifest-filter)

# Manifest modifier

*Manifest Modifier* is a work-in-progress project to modify video manifests.

Why? Video is a bit complex. Some manifests won't run on some devices because of the frame rate, or the bitrate, or other tags that may affect playback.

![manifest_modifier](/assets/manifest_modifier.png)

The image above is a perfect example that describes am usual problem: some devices can't play 60fps video. In order to solve this situation, we can use `manifest-modifier` to rewrite the manifest right before sending it to the users.

There are two ways to use this project, either as a lib or a server. This project is dividied into two crates: `manifest-filter` and `manifest-server`. `manifest-server` is a server built on top of [axum](https://github.com/tokio-rs/axum) and [m3u8-rs](https://github.com/rutgersc/m3u8-rs). It can be used without requiring advanced knowledge of the Rust programming language.

`manifest-filter` is the Rust code behind `manifest-server`. If you running your own server and can't use the `manifest-server`, no worries, you can use the same features by calling Rust code directly:

```rust
use manifest_filter::Master;
use std::io::Read;

let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
let mut content: Vec<u8> = Vec::new();
file.read_to_end(&mut content).unwrap();

let (_, master_playlist) = m3u8_rs::parse_master_playlist(&content).unwrap();
let mut master = Master {
    playlist: master_playlist,
};
master.filter_fps(Some(30.0));
 ```

The result should be  something like this

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio-aach-96",LANGUAGE="en",NAME="English",DEFAULT=YES,AUTOSELECT=YES,CHANNELS="2"
#EXT-X-STREAM-INF:BANDWIDTH=600000,AVERAGE-BANDWIDTH=600000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=384x216,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=249984.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=800000,AVERAGE-BANDWIDTH=800000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=768x432,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=1320960.m3u8
```

## Features

### Master playlist

**Bandwidth** - filter variants based on min and max values.

Request:

```
curl --request POST \
  --url 'http://localhost:3000/master?min_bitrate=800000&max_bitrate=2000000' \
  --header 'content-type: text/html; charset=UTF-8' \
  --header 'user-agent: vscode-restclient' \
  --data '< ../manifest-filter/manifests/master.m3u8'
```

Response:

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio-aach-96",LANGUAGE="en",NAME="English",DEFAULT=YES,AUTOSELECT=YES,CHANNELS="2"
#EXT-X-STREAM-INF:BANDWIDTH=800000,AVERAGE-BANDWIDTH=800000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=768x432,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=1320960.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=1500000,AVERAGE-BANDWIDTH=1500000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=1280x720,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=3092992.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2000000,AVERAGE-BANDWIDTH=2000000,CODECS="mp4a.40.5,avc1.640029",RESOLUTION=1920x1080,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=4686976.m3u8
```

<details>
<summary>Original playlist</summary>

As you can see, the original playlist was slightly different:

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio-aach-96",LANGUAGE="en",NAME="English",DEFAULT=YES,AUTOSELECT=YES,CHANNELS="2"
#EXT-X-STREAM-INF:BANDWIDTH=600000,AVERAGE-BANDWIDTH=600000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=384x216,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=249984.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=800000,AVERAGE-BANDWIDTH=800000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=768x432,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=1320960.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=1500000,AVERAGE-BANDWIDTH=1500000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=1280x720,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=3092992.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2000000,AVERAGE-BANDWIDTH=2000000,CODECS="mp4a.40.5,avc1.640029",RESOLUTION=1920x1080,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=4686976.m3u8
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=37000,CODECS="avc1.64001F",RESOLUTION=384x216,URI="keyframes/variant-video=249984.m3u8"
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=193000,CODECS="avc1.64001F",RESOLUTION=768x432,URI="keyframes/variant-video=1320960.m3u8"
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=296000,CODECS="avc1.64001F",RESOLUTION=1280x720,URI="keyframes/variant-video=2029952.m3u8"
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=684000,CODECS="avc1.640029",RESOLUTION=1920x1080,URI="keyframes/variant-video=4686976.m3u8"
```
</details>

**Frame rate** - filter variants based on a predefined *fps*:

Request:

```
curl --request POST \
  --url 'http://localhost:3000/master?rate=60' \
  --header 'content-type: text/html; charset=UTF-8' \
  --header 'user-agent: vscode-restclient' \
  --data '< ../manifest-filter/manifests/master.m3u8'
```

Response:

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio-aach-96",LANGUAGE="en",NAME="English",DEFAULT=YES,AUTOSELECT=YES,CHANNELS="2"
#EXT-X-STREAM-INF:BANDWIDTH=1500000,AVERAGE-BANDWIDTH=1500000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=1280x720,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=3092992.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2000000,AVERAGE-BANDWIDTH=2000000,CODECS="mp4a.40.5,avc1.640029",RESOLUTION=1920x1080,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=4686976.m3u8
```

<details>
<summary>Original playlist</summary>

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID="audio-aach-96",LANGUAGE="en",NAME="English",DEFAULT=YES,AUTOSELECT=YES,CHANNELS="2"
#EXT-X-STREAM-INF:BANDWIDTH=600000,AVERAGE-BANDWIDTH=600000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=384x216,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=249984.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=800000,AVERAGE-BANDWIDTH=800000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=768x432,FRAME-RATE=30,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=1320960.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=1500000,AVERAGE-BANDWIDTH=1500000,CODECS="mp4a.40.5,avc1.64001F",RESOLUTION=1280x720,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=3092992.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2000000,AVERAGE-BANDWIDTH=2000000,CODECS="mp4a.40.5,avc1.640029",RESOLUTION=1920x1080,FRAME-RATE=60,AUDIO="audio-aach-96",CLOSED-CAPTIONS=NONE
variant-audio_1=96000-video=4686976.m3u8
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=37000,CODECS="avc1.64001F",RESOLUTION=384x216,URI="keyframes/variant-video=249984.m3u8"
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=193000,CODECS="avc1.64001F",RESOLUTION=768x432,URI="keyframes/variant-video=1320960.m3u8"
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=296000,CODECS="avc1.64001F",RESOLUTION=1280x720,URI="keyframes/variant-video=2029952.m3u8"
#EXT-X-I-FRAME-STREAM-INF:BANDWIDTH=684000,CODECS="avc1.640029",RESOLUTION=1920x1080,URI="keyframes/variant-video=4686976.m3u8"
```
</details>

### Media playlist

**DVR** - remove segments (backwards) based on the duration (seconds)

```
curl --request POST \
  --url 'http://localhost:3000/media?dvr=15' \
  --header 'content-type: text/html; charset=UTF-8' \
  --header 'user-agent: vscode-restclient' \
  --data '< ../manifest-filter/manifests/media.m3u8'
```

Response:

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-INDEPENDENT-SEGMENTS
#EXT-X-TARGETDURATION:8
#EXT-X-MEDIA-SEQUENCE:320035373
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035703.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035702.ts
#EXT-X-PROGRAM-DATE-TIME:2020-09-15T14:01:39.133333+00:00
#EXT-X-CUE-IN
#EXTINF:5.8666, no desc
variant-audio_1=96000-video=249984-320035701.ts
```

<details>
<summary>Original playlist</summary>

```
#EXTM3U
#EXT-X-VERSION:4
#EXT-X-MEDIA-SEQUENCE:320035356
#EXT-X-INDEPENDENT-SEGMENTS
#EXT-X-TARGETDURATION:8
#EXT-X-PROGRAM-DATE-TIME:2020-09-15T13:32:55Z
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035684.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035685.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035686.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035687.ts
#EXTINF:4.1333, no desc
variant-audio_1=96000-video=249984-320035688.ts
#EXT-X-DATERANGE:ID="4026531847",START-DATE="2020-09-15T14:00:39.133333Z",PLANNED-DURATION=60,SCTE35-OUT=0xFC3025000000000BB800FFF01405F00000077FEFFE0AF311F0FE005265C0000101010000817C918E
#EXT-X-CUE-OUT:60
#EXT-X-PROGRAM-DATE-TIME:2020-09-15T14:00:39.133333Z
#EXTINF:5.8666, no desc
variant-audio_1=96000-video=249984-320035689.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035690.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035691.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035692.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035693.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035694.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035695.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035696.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035697.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035698.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035699.ts
#EXTINF:4.1333, no desc
variant-audio_1=96000-video=249984-320035700.ts
#EXT-X-CUE-IN
#EXT-X-PROGRAM-DATE-TIME:2020-09-15T14:01:39.133333Z
#EXTINF:5.8666, no desc
variant-audio_1=96000-video=249984-320035701.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035702.ts
#EXTINF:5, no desc
variant-audio_1=96000-video=249984-320035703.ts
```
</details>

## Tests

```
cargo test
```

## Lint

```
cargo clippy
```
