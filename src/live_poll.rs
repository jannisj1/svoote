use smartstring::{Compact, SmartString};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::Instant;
use uuid::Uuid;

use crate::app_error::AppError;
use crate::config::{LIVE_POLL_PARTICIPANT_LIMIT, POLL_EXIT_TIMEOUT};
use crate::live_poll_store::{ShortID, LIVE_POLL_STORE};
use crate::play::Player;
use crate::slide::Slide;

pub struct LivePoll {
    pub host_session_id: Uuid,
    pub slides: Vec<Slide>,
    pub player_indices: BTreeMap<Uuid, usize>,
    pub players: Vec<Player>,
    pub current_slide_index: usize,
    pub current_item_start_time: tokio::time::Instant,
    pub start_poll_channel_sender: Option<oneshot::Sender<()>>,
    pub set_slide_index_channel_sender: mpsc::Sender<usize>,
    pub slide_change_notification_channel_receiver: broadcast::Receiver<usize>,
    pub stats_change_notification_channel_sender: broadcast::Sender<usize>,
    pub stats_change_notification_channel_receiver: broadcast::Receiver<usize>,
    pub emoji_channel_sender: broadcast::Sender<(usize, SmartString<Compact>)>,
    pub emoji_channel_receiver: broadcast::Receiver<(usize, SmartString<Compact>)>,
    pub exit_poll_channel_sender: mpsc::Sender<()>,
    //pub leaderboard_enabled: bool,
    //pub allow_custom_player_names: bool,
}

impl LivePoll {
    pub fn orchestrate(
        slides: Vec<Slide>,
        host_session_id: Uuid,
        //leaderboard_enabled: bool,
        //allow_custom_player_names: bool,
    ) -> Result<(ShortID, Arc<Mutex<Self>>), AppError> {
        let (start_poll_channel_sender, start_poll_channel_receiver) = oneshot::channel::<()>();
        let (set_slide_index_channel_sender, mut set_slide_index_channel_receiver) =
            mpsc::channel(16);
        let (slide_change_notification_channel_sender, slide_change_notification_channel_receiver) =
            broadcast::channel(16);
        let (stats_change_notification_channel_sender, stats_change_notification_channel_receiver) =
            broadcast::channel(16);
        let (emoji_channel_sender, emoji_channel_receiver) = broadcast::channel(16);
        let (exit_poll_channel_sender, mut exit_poll_channel_receiver) = mpsc::channel(16);

        let (poll_id, live_poll) = LIVE_POLL_STORE.insert(LivePoll {
            host_session_id,
            slides,
            player_indices: BTreeMap::new(),
            players: Vec::new(),
            current_slide_index: 0usize,
            current_item_start_time: Instant::now(),
            start_poll_channel_sender: Some(start_poll_channel_sender),
            set_slide_index_channel_sender,
            slide_change_notification_channel_receiver,
            stats_change_notification_channel_sender: stats_change_notification_channel_sender
                .clone(),
            stats_change_notification_channel_receiver,
            emoji_channel_sender,
            emoji_channel_receiver,
            exit_poll_channel_sender,
            //leaderboard_enabled,
            //allow_custom_player_names,
        })?;

        let return_live_poll_handle = live_poll.clone();

        tokio::spawn(async move {
            let _live_poll_drop = RmLivePollOnDrop {
                poll_id,
                host_session_id,
            };
            let _ = start_poll_channel_receiver.await;

            loop {
                select! {
                    slide_index = set_slide_index_channel_receiver.recv() => {
                        if let Some(mut slide_index) = slide_index {
                            let mut live_poll = live_poll.lock().unwrap();
                            if slide_index >= live_poll.slides.len() {
                                slide_index = 0;
                            }

                            live_poll.current_slide_index = slide_index;
                            live_poll.current_item_start_time = Instant::now();

                            let _ = slide_change_notification_channel_sender.send(slide_index);
                            let _ = stats_change_notification_channel_sender.send(slide_index);
                        }
                    }
                    _ = exit_poll_channel_receiver.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(POLL_EXIT_TIMEOUT) => {
                        break;
                    }
                };
            }
        });

        return Ok((poll_id, return_live_poll_handle));
    }

    pub fn get_or_create_player(&mut self, player_session_id: &Uuid) -> Option<usize> {
        if let Ok(player_index) = self.get_player_index(player_session_id) {
            return Some(player_index);
        }

        if self.players.len() >= LIVE_POLL_PARTICIPANT_LIMIT {
            return None;
        }

        let new_player_idx = self.players.len();
        let new_player = Player::new(new_player_idx);

        self.player_indices
            .insert(player_session_id.clone(), new_player_idx);
        self.players.push(new_player);

        for item in &mut self.slides {
            item.add_player();
        }

        //let _ = self.ch_players_updated_send.send(());

        return Some(new_player_idx);
    }

    pub fn get_player_index(&self, player_session_id: &Uuid) -> Result<usize, AppError> {
        return self
            .player_indices
            .get(&player_session_id)
            .map(|index| *index)
            .ok_or(AppError::BadRequest(
                "Player with this auth token did not join the poll yet".to_string(),
            ));
    }

    pub fn get_player<'a>(&'a self, player_index: usize) -> &'a Player {
        return &self.players[player_index];
    }

    /*pub fn get_player_mut<'a>(&'a mut self, player_index: usize) -> &'a mut Player {
        return &mut self.players[player_index];
    }*/

    pub fn get_current_slide<'a>(&'a mut self) -> &'a mut Slide {
        return &mut self.slides[self.current_slide_index];
    }

    pub fn get_current_slide_start_time(&self) -> tokio::time::Instant {
        return self.current_item_start_time.clone();
    }
}

pub struct RmLivePollOnDrop {
    pub poll_id: ShortID,
    pub host_session_id: Uuid,
}

impl Drop for RmLivePollOnDrop {
    fn drop(&mut self) {
        LIVE_POLL_STORE.remove(&self.host_session_id, self.poll_id);
    }
}
