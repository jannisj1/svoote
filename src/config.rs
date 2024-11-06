pub const MAX_FREE_TEXT_ANSWERS: usize = 3;
pub const LIVE_POLL_PARTICIPANT_LIMIT: usize = 100usize;

pub const COLOR_PALETTE: &[&'static str] = &[
    "bg-rose-500",
    "bg-cyan-600",
    "bg-lime-500",
    "bg-fuchsia-600",
    "bg-slate-600",
    "bg-teal-500",
];

pub const POLL_MAX_MC_ANSWERS: usize = 6;
pub const POLL_MAX_SLIDES: usize = 32;
pub const POLL_MAX_STR_LEN: usize = 1024;

pub const POLL_EXIT_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(2 * 60 * 60); // 2 hours

pub const CUSTOM_PLAYER_NAME_LENGTH_LIMIT: usize = 32;
