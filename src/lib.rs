use m3u8_rs::{MasterPlaylist};

fn filter_fps(pl: MasterPlaylist, rate: f64) -> MasterPlaylist {
    let mut mpl = pl.clone();
    mpl.variants = pl.variants
        .into_iter()
        .filter( |v| v.frame_rate == Some(rate))
        .collect::<Vec<m3u8_rs::VariantStream>>();
    mpl
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use super::*;

    #[test]
    fn filter_30_fps() {
        let mut file = std::fs::File::open("manifests/master.m3u8").unwrap();
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).unwrap();

        if let Result::Ok((_, master_playlist)) =  m3u8_rs::parse_master_playlist(&content) {
            let nmp = filter_fps(master_playlist, 60.0);
            assert_eq!(nmp.variants.len(), 2);
        };
    }
}
