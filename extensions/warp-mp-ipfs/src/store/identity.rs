#![allow(dead_code)]
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

use futures::{SinkExt, StreamExt, TryFutureExt};
use ipfs::{Ipfs, IpfsPath, Keypair, Types};
use libipld::{ipld, Cid, Ipld};
use warp::{
    crypto::{rand::Rng, PublicKey},
    error::Error,
    multipass::identity::Identity,
    sync::{Arc, Mutex, RwLock},
    tesseract::Tesseract,
};

use super::{libp2p_pub_to_pub, topic_discovery, IDENTITY_BROADCAST};

#[derive(Clone)]
pub struct IdentityStore {
    ipfs: Ipfs<Types>,

    identity: Arc<RwLock<Option<Identity>>>,

    cache: Arc<RwLock<Vec<Identity>>>,

    start_event: Arc<AtomicBool>,

    end_event: Arc<AtomicBool>,

    tesseract: Tesseract,
}

impl Drop for IdentityStore {
    fn drop(&mut self) {
        self.disable_event();
        self.end_event();
    }
}

#[derive(Debug, Clone)]
pub enum LookupBy {
    PublicKey(PublicKey),
    Username(String),
}

impl IdentityStore {
    pub async fn new(
        ipfs: Ipfs<Types>,
        tesseract: Tesseract,
        discovery: bool,
        interval: u64,
    ) -> Result<Self, Error> {
        let cache = Arc::new(Default::default());
        let identity = Arc::new(Default::default());
        let start_event = Arc::new(Default::default());
        let end_event = Arc::new(Default::default());

        let store = Self {
            ipfs,
            cache,
            identity,
            start_event,
            end_event,
            tesseract,
        };

        if let Ok(ident) = store.own_identity().await {
            *store.identity.write() = Some(ident);
            store.start_event.store(true, Ordering::SeqCst);
        }
        let id_broadcast_stream = store
            .ipfs
            .pubsub_subscribe(IDENTITY_BROADCAST.into())
            .await?;
        let store_inner = store.clone();

        if discovery {
            let ipfs = store.ipfs.clone();
            tokio::spawn(async {
                if let Err(_e) = topic_discovery(ipfs, IDENTITY_BROADCAST).await {
                    //TODO: Log
                }
            });
        }

        tokio::spawn(async move {
            let store = store_inner;

            futures::pin_mut!(id_broadcast_stream);

            let mut tick = tokio::time::interval(Duration::from_millis(interval));
            loop {
                if store.end_event.load(Ordering::SeqCst) {
                    break;
                }
                if !store.start_event.load(Ordering::SeqCst) {
                    continue;
                }
                tokio::select! {
                    message = id_broadcast_stream.next() => {
                        if let Some(message) = message {
                            if let Ok(identity) = serde_json::from_slice::<Identity>(&message.data) {
                                if let Some(own_id) = store.identity.read().clone() {
                                    if own_id == identity {
                                        continue
                                    }
                                }

                                if store.cache.read().contains(&identity) {
                                    continue;
                                }

                                let index = store.cache.read()
                                    .iter()
                                    .position(|ident| ident.public_key() == identity.public_key());

                                if let Some(index) = index {
                                    store.cache.write().remove(index);
                                }

                                store.cache.write().push(identity);
                            }
                        }

                    }
                    _ = tick.tick() => {
                        //TODO: Add check to determine if peers are subscribed to topic before publishing
                        //TODO: Provide a signed and/or encrypted payload
                        let ident = store.identity.read().clone();
                        if let Some(ident) = ident.as_ref() {
                            if let Ok(bytes) = serde_json::to_vec(&ident) {
                                if let Err(_e) = store.ipfs.pubsub_publish(IDENTITY_BROADCAST.into(), bytes).await {
                                    continue
                                }
                            }
                        }
                    }
                }
            }
        });
        Ok(store)
    }

    fn cache(&self) -> Vec<Identity> {
        self.cache.read().clone()
    }

    pub async fn create_identity(&mut self, username: Option<&str>) -> Result<Identity, Error> {
        let raw_kp = self.get_raw_keypair()?;

        if self.own_identity().await.is_ok() {
            return Err(Error::IdentityExist);
        }

        let raw_kp = self.get_raw_keypair()?;

        let mut identity = Identity::default();
        let public_key = PublicKey::from_bytes(&raw_kp.public().encode());

        let username = match username {
            Some(u) => u.to_string(),
            None => warp::multipass::generator::generate_name(),
        };

        identity.set_username(&username);
        identity.set_short_id(warp::crypto::rand::thread_rng().gen_range(0, 9999));
        identity.set_public_key(public_key);

        // TODO: Convert our identity to ipld(?)
        let bytes = serde_json::to_vec(&identity)?;

        // Store the identity as a dag
        // TODO: Create a single root dag for the Cid
        let ident_cid = self.ipfs.put_dag(ipld!(bytes)).await?;
        let friends_cid = self.ipfs.put_dag(ipld!(Vec::<u8>::new())).await?;
        let block_cid = self.ipfs.put_dag(ipld!(Vec::<u8>::new())).await?;

        // Pin the dag
        self.ipfs.insert_pin(&ident_cid, false).await?;
        self.ipfs.insert_pin(&friends_cid, false).await?;
        self.ipfs.insert_pin(&block_cid, false).await?;

        // Note that for the time being we will be storing the Cid to tesseract,
        // however this may be handled a different way.
        // TODO: Provide the Cid to DHT
        self.tesseract.set("ident_cid", &ident_cid.to_string())?;
        self.tesseract
            .set("friends_cid", &friends_cid.to_string())?;
        self.tesseract.set("block_cid", &block_cid.to_string())?;

        self.update_identity().await?;
        self.enable_event();

        Ok(identity)
    }

    pub fn lookup(&self, lookup: LookupBy) -> Result<Identity, Error> {
        // Check own identity just in case since we dont store this in the cache
        if let Some(ident) = self.identity.read().clone() {
            match lookup {
                LookupBy::PublicKey(pubkey) if ident.public_key() == pubkey => return Ok(ident),
                LookupBy::Username(username) if ident.username() == username => return Ok(ident),
                _ => {}
            };
        }

        for ident in self.cache() {
            match &lookup {
                LookupBy::PublicKey(pubkey) if &ident.public_key() == pubkey => return Ok(ident),
                LookupBy::Username(username) if &ident.username() == username => return Ok(ident),
                _ => continue,
            }
        }
        Err(Error::IdentityDoesntExist)
    }

    pub fn get_keypair(&self) -> anyhow::Result<Keypair> {
        match self.tesseract.retrieve("ipfs_keypair") {
            Ok(keypair) => {
                let kp = bs58::decode(keypair).into_vec()?;
                let id_kp = warp::crypto::ed25519_dalek::Keypair::from_bytes(&kp)?;
                let secret =
                    libp2p::identity::ed25519::SecretKey::from_bytes(id_kp.secret.to_bytes())?;
                Ok(Keypair::Ed25519(secret.into()))
            }
            Err(_) => anyhow::bail!(Error::PrivateKeyInvalid),
        }
    }

    pub fn get_raw_keypair(&self) -> anyhow::Result<libp2p::identity::ed25519::Keypair> {
        match self.get_keypair()? {
            Keypair::Ed25519(kp) => Ok(kp),
            _ => anyhow::bail!("Unsupported keypair"),
        }
    }

    pub async fn own_identity(&self) -> Result<Identity, Error> {
        let identity = match self.tesseract.retrieve("ident_cid") {
            Ok(cid) => {
                let cid: Cid = cid.parse().map_err(anyhow::Error::from)?;
                let path = IpfsPath::from(cid);
                match self.ipfs.get_dag(path).await {
                    Ok(Ipld::Bytes(bytes)) => serde_json::from_slice::<Identity>(&bytes)?,
                    _ => return Err(Error::IdentityDoesntExist), //Note: It should not hit here unless the repo is corrupted
                }
            }
            Err(_) => return Err(Error::IdentityDoesntExist),
        };

        let public_key = identity.public_key();
        let kp_public_key = libp2p_pub_to_pub(&self.get_keypair()?.public())?;

        if public_key != kp_public_key {
            return Err(Error::IdentityDoesntExist);
        }

        Ok(identity)
    }

    pub async fn update_identity(&self) -> Result<(), Error> {
        let ident = self.own_identity().await?;
        *self.identity.write() = Some(ident);
        Ok(())
    }

    pub fn enable_event(&mut self) {
        self.start_event.store(true, Ordering::SeqCst);
    }

    pub fn disable_event(&mut self) {
        self.start_event.store(false, Ordering::SeqCst);
    }

    pub fn end_event(&mut self) {
        self.end_event.store(true, Ordering::SeqCst);
    }
}
