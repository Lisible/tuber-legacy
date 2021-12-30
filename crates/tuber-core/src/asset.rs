use crate::{CoreError, CoreResult};
use log::info;
use serde_derive::Deserialize;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::PathBuf;

const ASSETS_DIRECTORY: &'static str = "assets";
const ASSET_DESCRIPTION_FILE: &'static str = "asset.json";

pub type GenericLoader = Box<dyn Fn(&AssetMetadata) -> Box<dyn Any>>;

pub struct AssetStore {
    assets: HashMap<TypeId, HashMap<String, Box<dyn Any>>>,
    asset_loaders: HashMap<TypeId, GenericLoader>,
    assets_metadata: HashMap<String, AssetMetadata>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            asset_loaders: HashMap::new(),
            assets_metadata: HashMap::new(),
        }
    }

    pub fn load_assets_metadata(&mut self) -> CoreResult<()> {
        info!("Loading assets metadata");
        let paths = match std::fs::read_dir(ASSETS_DIRECTORY) {
            Ok(paths) => paths,
            Err(_) => return Ok(()),
        };

        let asset_directory_paths: Vec<_> = paths
            .filter_map(|p| p.ok())
            .filter(|p| p.path().is_dir())
            .collect();

        for asset_directory_path in asset_directory_paths {
            let mut path = asset_directory_path.path();
            path.push(ASSET_DESCRIPTION_FILE);

            if !path.is_file() {
                return Err(CoreError::AssetDescriptionFileNotFound);
            }

            let f = std::fs::File::open(path)
                .map_err(|e| CoreError::AssetDescriptionFileOpenError(e))?;
            let reader = BufReader::new(f);
            let mut asset_metadata: AssetMetadata = serde_json::from_reader(reader)
                .map_err(|e| CoreError::AssetDescriptionFileParseError(e))?;
            asset_metadata.asset_path = asset_directory_path.path().into();
            info!(
                "Loaded resource metadata identifier={} kind={}",
                &asset_metadata.identifier, &asset_metadata.kind
            );
            self.assets_metadata
                .insert(asset_metadata.identifier.clone(), asset_metadata);
        }

        info!("Assets metadata loading done.");
        Ok(())
    }

    pub fn register_loaders<Loader>(&mut self, loaders: Vec<(TypeId, Loader)>)
    where
        Loader: 'static + Fn(&AssetMetadata) -> Box<dyn Any>,
    {
        for (type_id, loader) in loaders {
            self.asset_loaders.insert(
                type_id,
                Box::new(move |asset_metadata: &AssetMetadata| ((loader)(asset_metadata))),
            );
        }
    }

    pub fn register_loader<AssetType, Loader>(&mut self, loader: Loader)
    where
        AssetType: 'static + Any,
        Loader: 'static + Fn(&AssetMetadata) -> Box<AssetType>,
    {
        self.asset_loaders.insert(
            TypeId::of::<AssetType>(),
            Box::new(move |asset_metadata: &AssetMetadata| ((loader)(asset_metadata))),
        );
    }

    pub fn has_asset<AssetType>(&self, identifier: &str) -> bool
    where
        AssetType: 'static + Any,
    {
        let type_id = TypeId::of::<AssetType>();
        self.assets.get(&type_id).is_some() && self.assets[&type_id].contains_key(identifier)
    }

    pub fn load<AssetType>(&mut self, identifier: &str) -> CoreResult<()>
    where
        AssetType: 'static + Any,
    {
        if self.has_asset::<AssetType>(identifier) {
            return Ok(());
        }

        let type_id = TypeId::of::<AssetType>();

        let asset_metadata = self
            .assets_metadata
            .get(identifier)
            .ok_or(CoreError::AssetMetadataNotFound)?;

        let asset_storage = self.assets.entry(type_id).or_insert(HashMap::new());
        asset_storage.insert(
            identifier.into(),
            (self
                .asset_loaders
                .get(&type_id)
                .ok_or(CoreError::AssetLoaderNotFound)?)(asset_metadata),
        );
        Ok(())
    }

    pub fn insert_asset<AssetType>(
        &mut self,
        asset_metadata: AssetMetadata,
        asset: AssetType,
    ) -> CoreResult<()>
    where
        AssetType: 'static + Any,
    {
        let type_id = TypeId::of::<AssetType>();
        let asset_storage = self.assets.entry(type_id).or_insert(HashMap::new());
        asset_storage.insert(asset_metadata.identifier.clone(), Box::new(asset));
        self.assets_metadata
            .insert(asset_metadata.identifier.clone(), asset_metadata);
        Ok(())
    }

    pub fn stored_asset<AssetType>(&self, identifier: &str) -> CoreResult<&AssetType>
    where
        AssetType: 'static + Any,
    {
        Ok(self
            .assets
            .get(&TypeId::of::<AssetType>())
            .ok_or(CoreError::AssetStorageNotFound)?
            .get(identifier)
            .ok_or(CoreError::AssetNotFound)?
            .as_ref()
            .downcast_ref()
            .ok_or(CoreError::AssetDowncastError)?)
    }

    pub fn asset<AssetType>(&mut self, identifier: &str) -> CoreResult<&AssetType>
    where
        AssetType: 'static + Any,
    {
        let type_id = TypeId::of::<AssetType>();

        let asset_storage = self.assets.get(&type_id);
        if asset_storage.is_none() || asset_storage.unwrap().get(identifier).is_none() {
            self.load::<AssetType>(identifier)?;
        }

        self.stored_asset::<AssetType>(identifier)
    }
}

pub trait IntoLoader<F> {
    fn into_loader(self) -> GenericLoader;
}

impl<F> IntoLoader<F> for F
where
    F: 'static + Fn(&AssetMetadata) -> Box<dyn Any>,
{
    fn into_loader(self) -> GenericLoader {
        Box::new(self)
    }
}

#[derive(Deserialize)]
pub struct AssetMetadata {
    pub identifier: String,
    pub kind: String,
    pub metadata: HashMap<String, String>,
    #[serde(skip)]
    pub asset_path: PathBuf,
}