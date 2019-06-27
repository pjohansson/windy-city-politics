use amethyst::{
    assets::{AssetStorage, Handle, Loader, Progress},
    renderer::{
        palette::{Pixel, Srgb},
        rendy::{
            resource::{Filter, SamplerInfo, ViewKind, WrapMode},
            texture::{pixel::Rgb8Srgb, TextureBuilder},
        },
        types::TextureData,
        Kind, Texture,
    },
};

pub fn create_texture<P: Progress>(
    data: &[[u8; 4]],
    (nx, ny): (u32, u32),
    store: &AssetStorage<Texture>,
    loader: &Loader,
    progress: P,
) -> Result<Handle<Texture>, String> {
    match data.len() {
        0 => Err(String::from(
            "input texture data size was 0 but cannot create texture from no data",
        )),
        n if n != (nx * ny) as usize => Err(format![
            "input texture data size {} does not match given dimensions {} x {} = {}",
            data.len(),
            nx,
            ny,
            nx * ny
        ]),
        _ => {
            let buffer: Vec<Rgb8Srgb> = data
                .iter()
                .map(|p| Srgb::from_raw(p))
                .map(|&p| p.into())
                .collect();

            let texture_data: TextureData = TextureBuilder::new()
                .with_data(buffer)
                .with_data_width(nx)
                .with_data_height(ny)
                .with_kind(Kind::D2(nx, ny, 1, 1))
                .with_view_kind(ViewKind::D2)
                .with_sampler_info(SamplerInfo::new(Filter::Nearest, WrapMode::Tile))
                .into();

            Ok(loader.load_from_data(texture_data, progress, &store))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use amethyst::assets::ProgressCounter;
    // use amethyst_test::prelude::*;
    use lazy_static::lazy_static;
    use rayon::ThreadPoolBuilder;
    use std::sync::Arc;

    lazy_static! {
        static ref LOADER: Loader = {
            let builder = ThreadPoolBuilder::new().num_threads(1);
            let pool = Arc::new(builder.build().expect("invalid config"));
            Loader::new("", pool)
        };
    }

    #[test]
    fn create_texture_with_matching_dimensions_returns_handle() {
        let data_1 = &[[0, 0, 0, 0]];
        let data_2 = &[[0, 0, 0, 0], [0, 0, 0, 0]];
        let data_6 = &[
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
        ];

        let store = AssetStorage::<Texture>::new();

        assert!(create_texture(data_1, (1, 1), &store, &LOADER, ()).is_ok());
        assert!(create_texture(data_2, (2, 1), &store, &LOADER, ()).is_ok());
        assert!(create_texture(data_2, (1, 2), &store, &LOADER, ()).is_ok());
        assert!(create_texture(data_6, (3, 2), &store, &LOADER, ()).is_ok());
    }

    #[test]
    fn create_texture_with_nonmatching_dimensions_returns_error() {
        let data_6 = &[
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
        ];

        let store = AssetStorage::<Texture>::new();

        assert!(create_texture(data_6, (1, 1), &store, &LOADER, ()).is_err());
        assert!(create_texture(data_6, (3, 3), &store, &LOADER, ()).is_err());
        assert!(create_texture(data_6, (0, 6), &store, &LOADER, ()).is_err());
        assert!(create_texture(data_6, (6, 0), &store, &LOADER, ()).is_err());
    }

    #[test]
    fn create_texture_with_no_data_returns_error() {
        let store = AssetStorage::<Texture>::new();
        assert!(create_texture(&[], (0, 0), &store, &LOADER, ()).is_err());
    }

    #[test]
    fn create_texture_with_progresscounter_updates_it() {
        let data = &[[0, 0, 0, 0]];

        let store = AssetStorage::<Texture>::new();
        let mut progress = ProgressCounter::new();

        let num_created = 4;

        for _ in 0..num_created {
            create_texture(data, (1, 1), &store, &LOADER, &mut progress).unwrap();
        }

        assert_eq!(
            num_created,
            progress.num_assets(),
            "progress counter was not updated when creating a texture"
        );
    }
}
