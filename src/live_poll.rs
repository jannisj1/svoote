use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use tokio::select;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::Instant;
use uuid::Uuid;

use crate::app_error::AppError;
use crate::auth_token::AuthToken;
use crate::config::LIVE_POLL_PARTICIPANT_LIMIT;
use crate::live_item::LiveItem;
use crate::live_poll_store::{ShortID, LIVE_POLL_STORE};
use crate::play::Player;
use crate::polls::PollV1;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub question: String,
    pub answers: Answers,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Answers {
    SingleChoice(Vec<(String, bool)>),
    FreeText(usize, Vec<String>), // Max answers per User, correct answers
}

pub struct LivePoll {
    pub host_auth_token: AuthToken,
    pub items: Vec<LiveItem>,
    pub player_indices: BTreeMap<Uuid, usize>,
    pub players: Vec<Player>,
    pub current_item_idx: usize,
    current_item_start_time: tokio::time::Instant,
    pub ch_start_signal: Option<oneshot::Sender<()>>,
    pub ch_players_updated_send: watch::Sender<()>,
    pub ch_players_updated_recv: watch::Receiver<()>,
    pub ch_question_state: watch::Receiver<QuestionAreaState>,
    pub ch_question_statistics_send: watch::Sender<QuestionStatisticsState>,
    pub ch_question_statistics_recv: watch::Receiver<QuestionStatisticsState>,
    pub ch_next_question: mpsc::Sender<()>,
    pub ch_previous_question: mpsc::Sender<()>,
    pub leaderboard_enabled: bool,
}

impl LivePoll {
    pub fn new(
        poll: PollV1,
        leaderboard_enabled: bool,
        auth_token: AuthToken,
    ) -> Result<(ShortID, Arc<Mutex<Self>>), AppError> {
        let (send_start_signal, recv_start_signal) = oneshot::channel::<()>();
        let (sse_host_question_send, sse_host_question_recv) =
            watch::channel(QuestionAreaState::None);
        let (send_question_statistics, recv_question_statistics) =
            watch::channel(QuestionStatisticsState::None);
        let (send_players_updated, recv_players_updated) = watch::channel(());
        let (send_next_question, mut recv_next_question) = mpsc::channel(4);
        let (send_previous_question, mut recv_previous_question) = mpsc::channel(4);

        let live_items: Vec<LiveItem> = poll
            .items
            .into_iter()
            .map(|item| LiveItem::new(item))
            .collect();

        let (poll_id, live_poll) = LIVE_POLL_STORE.insert(LivePoll {
            host_auth_token: auth_token,
            items: live_items,
            player_indices: BTreeMap::new(),
            players: Vec::new(),
            current_item_idx: 0usize,
            current_item_start_time: Instant::now(),
            ch_start_signal: Some(send_start_signal),
            ch_players_updated_send: send_players_updated,
            ch_players_updated_recv: recv_players_updated,
            ch_question_statistics_send: send_question_statistics,
            ch_question_statistics_recv: recv_question_statistics,
            ch_next_question: send_next_question,
            ch_previous_question: send_previous_question,
            ch_question_state: sse_host_question_recv,
            leaderboard_enabled,
        })?;

        let return_live_poll = live_poll.clone();

        tokio::spawn(async move {
            let _lq_drop = RmLqOnDrop(poll_id);
            let _ = recv_start_signal.await;

            let mut question_idx = 0usize;
            let mut is_last_question;

            loop {
                {
                    let mut live_poll = live_poll.lock().unwrap();
                    if question_idx >= live_poll.items.len() {
                        break;
                    }

                    is_last_question = question_idx + 1 == live_poll.items.len();

                    live_poll.current_item_idx = question_idx;
                    live_poll.current_item_start_time = Instant::now();

                    let _ = live_poll
                        .ch_question_statistics_send
                        .send(QuestionStatisticsState::Item(question_idx));
                }

                let _ = sse_host_question_send.send(QuestionAreaState::Item {
                    item_idx: question_idx,
                    is_last_question,
                });

                select! {
                    _ = recv_next_question.recv() => {
                        question_idx += 1;
                    }
                    _ = recv_previous_question.recv() => {
                        if question_idx > 0 {
                            question_idx -= 1;
                        }
                    }
                };
            }

            let _ = sse_host_question_send.send(QuestionAreaState::PollFinished);
            let _ = live_poll
                .lock()
                .unwrap()
                .ch_question_statistics_send
                .send(QuestionStatisticsState::None);

            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

            let _ = sse_host_question_send.send(QuestionAreaState::CloseSSE);
            let _ = live_poll
                .lock()
                .unwrap()
                .ch_question_statistics_send
                .send(QuestionStatisticsState::CloseSSE);
        });

        return Ok((poll_id, return_live_poll));
    }

    pub fn get_or_create_player(&mut self, auth_token: &AuthToken) -> Option<usize> {
        if let Ok(player_index) = self.get_player_index(auth_token) {
            return Some(player_index);
        }

        if self.players.len() >= LIVE_POLL_PARTICIPANT_LIMIT {
            return None;
        }

        let new_player_idx = self.players.len();
        let new_player = Player::new(new_player_idx);

        self.player_indices
            .insert(auth_token.token.clone(), new_player_idx);
        self.players.push(new_player);

        for item in &mut self.items {
            item.add_player();
        }

        let _ = self.ch_players_updated_send.send(());

        return Some(new_player_idx);
    }

    pub fn get_player_index(&self, auth_token: &AuthToken) -> Result<usize, AppError> {
        return self
            .player_indices
            .get(&auth_token.token)
            .map(|index| *index)
            .ok_or(AppError::BadRequest(
                "Player with this auth token did not join the poll yet".to_string(),
            ));
    }

    pub fn get_player<'a>(&'a self, player_index: usize) -> &'a Player {
        return &self.players[player_index];
    }

    pub fn get_current_item<'a>(&'a mut self) -> &'a mut LiveItem {
        return &mut self.items[self.current_item_idx];
    }

    pub fn get_current_item_start_time(&self) -> tokio::time::Instant {
        return self.current_item_start_time.clone();
    }
}

#[derive(Clone)]
pub enum QuestionAreaState {
    None,
    Item {
        item_idx: usize,
        is_last_question: bool,
    },
    PollFinished,
    CloseSSE,
}

#[derive(Clone)]
pub enum QuestionStatisticsState {
    None,
    Item(usize),
    CloseSSE,
}

pub struct RmLqOnDrop(pub ShortID);

impl Drop for RmLqOnDrop {
    fn drop(&mut self) {
        LIVE_POLL_STORE.remove(self.0);
    }
}
