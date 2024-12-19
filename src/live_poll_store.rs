use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

use crate::{app_error::AppError, live_poll::LivePoll};

pub type ShortID = u32;

pub static LIVE_POLL_STORE: LivePollStore = LivePollStore::new();

pub struct LivePollStore {
    pub polls: Mutex<BTreeMap<ShortID, Arc<Mutex<LivePoll>>>>,
    pub session_lookup: Mutex<BTreeMap<Uuid, ShortID>>,
}

impl LivePollStore {
    pub const fn new() -> Self {
        return LivePollStore {
            polls: Mutex::new(BTreeMap::new()),
            session_lookup: Mutex::new(BTreeMap::new()),
        };
    }

    pub fn get(&self, id: ShortID) -> Option<Arc<Mutex<LivePoll>>> {
        return self
            .polls
            .lock()
            .unwrap()
            .get(&id)
            .map(|live_poll| live_poll.clone());
    }

    pub fn insert(&self, live_poll: LivePoll) -> Result<(ShortID, Arc<Mutex<LivePoll>>), AppError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let host_session_id = live_poll.host_session_id.clone();

        let live_poll = Arc::new(Mutex::new(live_poll));
        let mut polls = self.polls.lock().unwrap();

        let random_id = (0..1000).map(|_| rng.gen_range::<ShortID, _>(1000..10_000)).find(|id| !polls.contains_key(&id))
            .unwrap_or((0..1000).map(|_| rng.gen_range::<ShortID, _>(10_000..1_000_000)).find(|id| !polls.contains_key(&id))
        .ok_or(AppError::OtherInternalServerError("Could not find a short id (between 1000 and 999 999 while creating a new live quiz."
                .to_string()))?);

        polls.insert(random_id, live_poll.clone());
        self.session_lookup
            .lock()
            .unwrap()
            .insert(host_session_id, random_id);

        return Ok((random_id, live_poll));
    }

    pub fn remove(&self, host_session_id: &Uuid, id: ShortID) {
        self.polls.lock().unwrap().remove(&id);
        self.session_lookup.lock().unwrap().remove(host_session_id);
    }

    pub fn get_by_session_id(
        &self,
        host_session_id: &Uuid,
    ) -> Option<(ShortID, Arc<Mutex<LivePoll>>)> {
        let poll_id = match self.session_lookup.lock().unwrap().get(host_session_id) {
            Some(poll_id) => *poll_id,
            None => return None,
        };

        if let Some(live_poll) = self.get(poll_id) {
            return Some((poll_id, live_poll));
        } else {
            return None;
        }
    }
}
