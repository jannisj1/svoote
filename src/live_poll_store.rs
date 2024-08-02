use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tower_sessions::Session;

use crate::{app_error::AppError, auth_token::AuthToken, live_poll::LivePoll};

pub type ShortID = u32;

pub static LIVE_POLL_STORE: LivePollStore = LivePollStore::new();

pub struct LivePollStore {
    inner: Mutex<BTreeMap<ShortID, Arc<Mutex<LivePoll>>>>,
}

impl LivePollStore {
    pub const fn new() -> Self {
        return LivePollStore {
            inner: Mutex::new(BTreeMap::new()),
        };
    }

    pub fn get(&self, id: ShortID) -> Option<Arc<Mutex<LivePoll>>> {
        return self.inner.lock().unwrap().get(&id).map(|lq| lq.clone());
    }

    pub fn insert(&self, lq: LivePoll) -> Result<(ShortID, Arc<Mutex<LivePoll>>), AppError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let lq = Arc::new(Mutex::new(lq));
        let mut polls = self.inner.lock().unwrap();

        let random_id = (0..1000).map(|_| rng.gen_range::<ShortID, _>(1000..10_000)).find(|id| !polls.contains_key(&id))
            .unwrap_or((0..1000).map(|_| rng.gen_range::<ShortID, _>(10_000..1_000_000)).find(|id| !polls.contains_key(&id))
        .ok_or(AppError::OtherInternalServerError("Could not find a short id (between 1000 and 999 999 while creating a new live quiz."
                .to_string()))?);

        polls.insert(random_id, lq.clone());

        return Ok((random_id, lq));
    }

    pub fn remove(&self, id: ShortID) {
        self.inner.lock().unwrap().remove(&id);
    }

    pub async fn get_from_session(
        &self,
        session: &Session,
        auth_token: &AuthToken,
    ) -> Result<Option<(ShortID, Arc<Mutex<LivePoll>>)>, AppError> {
        if let Some(poll_id) = session
            .get::<ShortID>("live_poll_id")
            .await
            .map_err(|e| AppError::DatabaseError(e))?
        {
            if let Some(lq) = self.get(poll_id) {
                if lq.lock().unwrap().host_auth_token.token == auth_token.token {
                    return Ok(Some((poll_id, lq)));
                }
            }
        }

        return Ok(None);
    }
}
