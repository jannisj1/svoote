pub const FREE_TEXT_MAX_CHAR_LENGTH: usize = 32;
pub const LIVE_POLL_PARTICIPANT_LIMIT: usize = 100usize;

pub const COLOR_PALETTE: &[&'static str] = &[
    "bg-rose-500",
    "bg-cyan-600",
    "bg-lime-600",
    "bg-fuchsia-600",
    "bg-slate-600",
    "bg-teal-600",
];

pub const COLOR_PALETTE_RGB: &[&'static str] = &[
    "#f43f5e", "#0891b2", "#65a30d", "#c026d3", "#475569", "#0d9488",
];

pub const POLL_MAX_MC_ANSWERS: usize = 6;
pub const POLL_MAX_SLIDES: usize = 32;
//pub const POLL_MAX_STR_LEN: usize = 1024;

pub const POLL_EXIT_TIMEOUT: tokio::time::Duration = tokio::time::Duration::from_secs(2 * 60 * 60); // 2 hours
pub const STATS_UPDATE_THROTTLE: tokio::time::Duration = tokio::time::Duration::from_secs(2);
