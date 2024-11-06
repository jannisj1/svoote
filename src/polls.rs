use axum::response::{IntoResponse, Response};
use maud::html;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{
    app_error::AppError,
    auth_token::AuthToken,
    config::{COLOR_PALETTE, POLL_MAX_MC_ANSWERS, POLL_MAX_SLIDES, POLL_MAX_STR_LEN},
    host,
    html_page::{self, render_header},
    illustrations::Illustrations,
    live_poll::{Answers, Item, LivePoll},
    live_poll_store::LIVE_POLL_STORE,
    static_file,
    svg_icons::SvgIcon,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistablePoll {
    pub items: Vec<Item>,
    pub version: usize,
    pub leaderboard_enabled: bool,
    pub question_time_limit_seconds: Option<usize>,
    pub allow_custom_names: bool,
}

impl PersistablePoll {
    pub fn new() -> Self {
        return PersistablePoll {
            items: Vec::new(),
            version: 1,
            leaderboard_enabled: false,
            question_time_limit_seconds: None,
            allow_custom_names: true,
        };
    }

    pub async fn from_session(session: &Session) -> Result<Self, AppError> {
        return Ok(session
            .get::<PersistablePoll>("poll_v1")
            .await
            .map_err(|e| AppError::DatabaseError(e))?
            .unwrap_or(PersistablePoll::new()));
    }

    pub async fn save_to_session(&self, session: &Session) -> Result<(), AppError> {
        return Ok(session
            .insert("poll_v1", self)
            .await
            .map_err(|e| AppError::DatabaseError(e))?);
    }
}

pub async fn post_start_poll(session: Session) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;

    let (poll_id, _live_poll) = match LIVE_POLL_STORE
        .get_from_session(&session, &auth_token)
        .await?
    {
        Some((poll_id, live_poll)) => (poll_id, live_poll),
        None => {
            let poll = PersistablePoll::from_session(&session).await?;

            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

            let (poll_id, live_poll) = LivePoll::orchestrate(poll.clone(), auth_token)?;

            session
                .insert("live_poll_id", poll_id)
                .await
                .map_err(|e| AppError::DatabaseError(e))?;

            live_poll
                .lock()
                .unwrap()
                .ch_start_signal
                .take()
                .map(|signal| {
                    let _ = signal.send(());
                });

            (poll_id, live_poll)
        }
    };

    return host::render_live_host(poll_id).await;
}

pub async fn get_poll_page(session: Session) -> Result<Response, AppError> {
    let auth_token = AuthToken::get_or_create(&session).await?;

    match LIVE_POLL_STORE
        .get_from_session(&session, &auth_token)
        .await?
    {
        Some((poll_id, _live_poll)) => return host::render_live_host(poll_id).await,
        None => {
            let poll = PersistablePoll::from_session(&session).await?;
            return render_edit_page(session, poll, 0).await;
        }
    };
}

async fn render_edit_page(
    session: Session,
    mut poll: PersistablePoll,
    active_item_index: usize,
) -> Result<Response, AppError> {
    if poll.items.len() == 0 {
        poll.items.push(Item {
            question: String::new(),
            answers: Answers::Untyped,
        });

        poll.save_to_session(&session).await?;
    }

    return Ok(html_page::render_html_page("Svoote", html! {
        #pollEditingArea ."mb-16" {
            (render_header(html! {}))
            div x-data="poll" {
                div ."mb-4 grid grid-cols-3" {
                    div x-data="{ open: false }" ."relative" {
                        button "@click"="open = !open" ."size-6 text-slate-400 hover:text-slate-600" { (SvgIcon::Settings.render()) }
                        div x-show="open" "@click.outside"="open = false" ."absolute left-0 top-8 w-64 h-fit z-20 p-4 text-left bg-white border rounded-lg shadow-lg" {
                            ."mb-3 text-xl font-semibold text-slate-700" { "Options" }
                            label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                input type="checkbox" x-model="poll.leaderboardEnabled" ."size-4 accent-indigo-500";
                                "Leaderboard"
                            }
                            ."ml-6 mb-3 text-slate-400 text-sm" { "Participants will receive points for submitting the correct answer. Faster responses get more points." }
                            label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                input type="checkbox" x-model="poll.allowCustomNames" ."size-4 accent-indigo-500";
                                "Custom names"
                            }
                            ."ml-6 mb-3 text-slate-400 text-sm" { "Allow participants to set a custom name." }
                            a download="poll.json" ":href"="'data:application/json;charset=utf-8,' + JSON.stringify(poll)" ."mb-3 flex gap-2 items-center text-slate-600 font-semibold" {
                                ."size-4" { (SvgIcon::Download.render()) }
                                "Save poll (.json)"
                            }
                            button ."flex gap-2 items-center text-slate-600 font-semibold" {
                                ."size-4" { (SvgIcon::UploadCloud.render()) }
                                "Import poll (.json)"
                            }
                        }
                    }
                    div {}
                    div ."flex justify-end" {
                        button "@click"="startPoll()" ."flex items-center gap-2" {
                            "Start"
                            ."size-5 text-slate-500" { (SvgIcon::Play.render()) }
                        }
                    }
                }
                div ."flex gap-6 overflow-hidden" {
                    button "@click"="poll.activeSlide -= 1; save();" ":disabled"="poll.activeSlide == 0" ."z-10 relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700 disabled:bg-slate-300" {
                        ."absolute inset-0 size-full flex items-center justify-center" {
                            ."size-4 translate-x-[-0.05rem]" { (SvgIcon::ChevronLeft.render()) }
                        }
                    }
                    div ."flex-1 relative h-[36rem]" {
                        template x-for="(slide, slide_index) in poll.slides" {
                            div ."absolute inset-0 size-full px-16 py-10 border rounded transition duration-500 ease-out transform-gpu" ":class"="slide_index == poll.activeSlide && 'shadow-xl'"
                                ":style"="'transform: perspective(100px) translateX(' + ((slide_index - poll.activeSlide) * 106) + '%) translateZ(' + (slide_index == poll.activeSlide ? '0' : '-10')  + 'px)'" {
                                input type="text" x-model="slide.question" "@input"="save" x-show="slide_index != 0 && slide_index != poll.slides.length - 1"
                                    "@keyup.enter"="let e = document.getElementById('s-' + slide_index + '-mc-answer-0'); if (e !== null) e.focus(); else document.getElementById('add-mc-answer-' + slide_index).click();"
                                    placeholder="Question"
                                    ."w-full mb-6 text-xl text-slate-800 outline-none";
                                template x-if="slide.type == 'undefined'" {
                                    div {
                                        ."mb-2 text-slate-500 tracking-tight text-center" {
                                            "Choose item type:"
                                        }
                                        ."flex justify-center gap-4" {
                                            button "@click"="slide.type = 'mc'" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                            {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) }
                                                "Multiple choice"
                                            }
                                            button "@click"="slide.type = 'ft'" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                            {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                                                "Free text"
                                            }
                                        }
                                    }
                                }
                                template x-if="slide.type == 'firstSlide'" {
                                    div ."h-full flex justify-center items-center gap-20" {
                                        div ."" {
                                            ."mb-1 text-sm text-slate-500 text-center" {
                                                "Enter on " a ."text-indigo-500 underline" href="" { "svoote.com" }
                                            }
                                            ."mb-6 text-3xl tracking-wider font-bold text-slate-700 text-center" {
                                                "1234"
                                            }
                                            ."w-lg flex justify-center" { " QR-Code" }
                                        }
                                        div ."w-[25rem]" {
                                            (Illustrations::TeamCollaboration.render())
                                        }
                                    }
                                }
                                template x-if="slide.type == 'lastSlide'" {
                                    div ."h-full flex flex-col justify-center" {
                                        ."mx-auto mt-6 w-24" { (Illustrations::InLove.render()) }
                                        ."mt-8 text-slate-500 text-center text-sm" { "This poll has no more items. Thank you for using svoote.com" }
                                    }
                                }
                                template x-if="slide.type == 'mc'" {
                                    div {
                                        template x-for="(answer, answer_index) in slide.mcAnswers" {
                                            div ."mb-4 flex items-center gap-3" {
                                                div x-text="incrementChar('A', answer_index)" ."text-slate-500" {}
                                                input type="text" x-model="answer.text" "@input"="save()"
                                                    "@keyup.enter"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[1].focus(); else next.click();"
                                                    ":id"="(answer_index == 0) && 's-' + slide_index + '-mc-answer-0'" ":class"="answer.isCorrect ? 'ring-[3px] ring-green-600' : 'ring-2 ring-slate-400 focus:ring-slate-500'"
                                                    ."w-full px-3 py-2 text-slate-700 rounded-lg outline-none";
                                                button "@click"="answer.isCorrect = !answer.isCorrect; save()" ":class"="answer.isCorrect ? 'text-green-600' : 'text-slate-300 hover:text-green-600'" ."size-6" { (SvgIcon::CheckSquare.render()) }
                                                button "@click"="slide.mcAnswers.splice(answer_index, 1); save();" ."size-6 text-slate-300 hover:text-slate-500" { (SvgIcon::Trash2.render()) }
                                            }
                                        }
                                        button
                                            "@click"={"if (slide.mcAnswers.length < " (POLL_MAX_MC_ANSWERS) ") { slide.mcAnswers.push({ text: '', isCorrect: false }); save(); $nextTick(() => $el.previousSibling.children[1].focus()); }" }
                                            ":class"={ "(slide.mcAnswers.length >= " (POLL_MAX_MC_ANSWERS) ") && 'hidden'" }
                                            ."ml-6 text-slate-700 underline"
                                            ":id"="'add-mc-answer-' + slide_index"
                                        {
                                            "Add answer"
                                        }
                                    }
                                }
                                template x-if="slide.type == 'ft'" {
                                    div {
                                        div ."pl-2 flex gap-2 items-center text-slate-500" {
                                            div ."size-4 shrink-0" { (SvgIcon::Edit3.render()) }
                                            "Free text: Participants can submit their own answer."
                                        }
                                        div ."mt-24 text-slate-500 text-center text-sm"  { "Correct answers:" }
                                        div ."mx-auto my-4 max-w-2xl flex justify-center flex-wrap gap-4" {
                                            template x-for="(answer, answer_index) in slide.ftAnswers" {
                                                div ."flex items-center gap-1" {
                                                    span x-init="$el.innerText = answer.text" "@input"="answer.text = $el.innerText; save();" contenteditable
                                                        ."block w-fit min-w-24 px-3 py-0.5 bg-slate-100 text-slate-500 rounded-full outline-none"
                                                        "@keydown.enter.prevent"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[0].focus(); else next.click();" {}
                                                    button "@click"="slide.ftAnswers.splice(answer_index, 1); save();" ."size-4 text-slate-300" { (SvgIcon::X.render()) }
                                                }
                                            }
                                            button "@click"="slide.ftAnswers.push({ text: '' }); save(); $nextTick(() => $el.previousSibling.children[0].focus());" ."size-7 p-0.5 text-slate-300 border rounded-full" { (SvgIcon::Plus.render()) }
                                        }
                                        div ."text-slate-500 text-center text-sm"  { "If the leaderboard is enabled, Participants can receive points for submitting the correct answer." }
                                    }
                                }
                            }
                        }
                    }
                    button x-show="poll.activeSlide + 1 < poll.slides.length" "@click"="poll.activeSlide += 1; save();" ."relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700" {
                        ."absolute inset-0 size-full flex items-center justify-center" {
                            ."size-4" { (SvgIcon::ChevronRight.render()) }
                        }
                    }
                    button x-ref="btnAddSlide" x-show="poll.activeSlide + 1 == poll.slides.length" "@click"="poll.slides.splice(poll.slides.length - 1, 0, createSlide(null)); $nextTick(() => { poll.activeSlide = poll.slides.length - 2; save() });"
                        ":disabled"={ "poll.slides.length >= " (POLL_MAX_SLIDES) }
                        ."relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700 disabled:bg-slate-300"
                    {
                        ."absolute inset-0 size-full flex items-center justify-center" {
                            ."size-4" { (SvgIcon::Plus.render()) }
                        }
                    }
                }
                div ."mt-4 flex justify-center gap-4" {
                    button "@click"="$refs.btnAddSlide.click();" ":disabled"={ "poll.slides.length >= " (POLL_MAX_SLIDES) }
                        ."group px-2 py-1 flex items-center gap-1 text-slate-500 border border-slate-300 rounded-full hover:bg-slate-100 disabled:text-slate-300 disabled:bg-transparent"
                    {
                        ."size-4 text-slate-500 group-disabled:text-slate-300" { (SvgIcon::Plus.render()) }
                        "Add slide"
                    }
                    button "@click"="poll.slides.splice(poll.activeSlide, 1); if (poll.activeSlide > 0) poll.activeSlide -= 1; save();" ":disabled"="poll.activeSlide == 0 || poll.activeSlide == poll.slides.length - 1"
                        ."group px-2 py-1 flex items-center gap-1 text-slate-500 border border-slate-300 rounded-full hover:bg-slate-100 disabled:text-slate-300 disabled:bg-transparent"
                    {
                        ."size-4 text-red-400 group-disabled:text-slate-300" { (SvgIcon::Trash2.render()) }
                        "Delete slide"
                    }
                }
            }
        }
        script src=(static_file::get_path("alpine.js")) {}
    }, true).into_response());
}
