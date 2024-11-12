use std::sync::{Arc, Mutex};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};

use axum_extra::extract::CookieJar;
use maud::html;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use smartstring::{Compact, SmartString};
use tokio::select;

use crate::{
    app_error::AppError,
    config::{COLOR_PALETTE, POLL_MAX_MC_ANSWERS, POLL_MAX_SLIDES},
    html_page::{self, render_header},
    live_poll::LivePoll,
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    session_id,
    slide::{FreeTextLiveAnswers, MultipleChoiceLiveAnswers, Slide, SlideType},
    static_file,
    svg_icons::SvgIcon,
};

pub async fn get_poll_page(cookies: CookieJar) -> Result<Response, AppError> {
    let (session_id, cookies) = session_id::get_or_create_session_id(cookies);

    let poll_is_live = LIVE_POLL_STORE.get_by_session_id(&session_id).is_some();

    let html = html_page::render_html_page(
        "Svoote",
        html! {
            script src=(static_file::get_path("qrcode.js")) {}
            @if poll_is_live { script { "document.pollAlreadyLive = true;" } }
            (render_header(html! {}))
            div x-data="poll" {
                div ."mx-16 mb-4 grid grid-cols-3 items-center" {
                    div ."flex items-center gap-2" {
                        div x-data="{ open: false }" ."relative size-[1.4rem]" {
                            button "@click"="open = !open" ."size-[1.4rem] text-slate-400 hover:text-slate-600" { (SvgIcon::Settings.render()) }
                            div x-show="open" x-cloak "@click.outside"="open = false" ."absolute left-0 top-8 w-64 h-fit z-20 p-4 text-left bg-white border rounded-lg shadow-lg" {
                                ."mb-3 text-xl font-semibold text-slate-700" { "Options" }
                                label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                    input type="checkbox" x-model="poll.enableLeaderboard" ."size-4 accent-indigo-500";
                                    "Leaderboard"
                                }
                                ."ml-6 mb-3 text-slate-400 text-sm" { "Participants will receive points for submitting the correct answer. Faster responses get more points." }
                                label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                    input type="checkbox" x-model="poll.allowCustomNames" ."size-4 accent-indigo-500";
                                    "Custom names"
                                }
                                ."ml-6 mb-3 text-slate-400 text-sm" { "Allow participants to set a custom name." }
                                hr ."mb-3";
                                a download="poll.json" ":href"="'data:application/json;charset=utf-8,' + JSON.stringify(poll)"
                                    ."mb-2 flex gap-2 items-center text-slate-600"
                                {
                                    ."size-4" { (SvgIcon::Save.render()) }
                                    "Save poll (.json)"
                                }
                                button ."mb-3 flex gap-2 items-center text-slate-600" {
                                    ."size-4" { (SvgIcon::Folder.render()) }
                                    "Load poll (.json)"
                                }
                                hr ."mb-3";
                                button "@click"="reset()" ":disabled"="isLive" ."flex gap-2 items-center text-slate-600 disabled:text-slate-300" {
                                    ."size-4" { (SvgIcon::Refresh.render()) }
                                    "Reset slides and settings"
                                }
                            }
                        }
                        button "@click"="gridView = !gridView"
                            ."size-6"
                            ":class"="!gridView ? 'text-slate-400 hover:text-slate-600' : 'text-indigo-500'"
                            { (SvgIcon::Grid.render()) }
                        button "@click"="poll.slides.splice(poll.slides.length, 0, createSlide('undefined')); $nextTick(() => { gotoSlide(poll.slides.length - 1) });"
                            ":disabled"={ "poll.slides.length >= " (POLL_MAX_SLIDES) }
                            ."-translate-x-1 size-6 text-slate-400 hover:text-slate-600 disabled:text-slate-400"
                            { (SvgIcon::Plus.render()) }
                    }
                    div {
                        div ."relative" {
                            template x-for="i in poll.slides.length" {
                                button x-text="i"
                                    ."absolute -top-3 left-1/2 size-6 rounded-full text-sm font-mono transition-transform duration-500 ease-out disabled:opacity-0"
                                    ":style"="`transform: translateX(${ ((i - 1) - poll.activeSlide) * 24 }px);`"
                                    ":class"="(i - 1 == poll.activeSlide ? 'bg-slate-500 text-slate-50' : 'bg-white text-slate-500')"
                                    ":disabled"="Math.abs((i - 1) - poll.activeSlide) > 6"
                                    "@click"="gotoSlide(i - 1)"
                                { }
                            }
                        }
                    }
                    div ."flex justify-end" {
                        button x-show="!isLive" "@click"="startPoll()" ."p-2 text-slate-50 bg-green-500 rounded-full hover:bg-green-600" {
                            ."size-5 translate-x-0.5" { (SvgIcon::Play.render()) }
                        }
                        button x-show="isLive" x-cloak "@click"="stopPoll()" ."p-3 text-slate-50 bg-red-500 rounded-full hover:bg-red-700" {
                            ."size-3 bg-slate-50" {}
                        }
                    }
                }
                div ."px-12 overflow-x-hidden overflow-y-scroll" {
                    div ."relative h-[36rem]" {
                        template x-for="(slide, slide_index) in poll.slides" {
                            div ."absolute inset-0 size-full px-16 py-10 bg-white border rounded ring-0 ring-indigo-500 transition-transform duration-500 ease-out transform-gpu"
                                ":class"="slide_index == poll.activeSlide ? (gridView ? 'ring-4 shadow-xl' : 'shadow-xl') : (gridView ? 'cursor-pointer hover:ring-4 hover:shadow-xl' : 'cursor-pointer')"
                                "@click"="if (slide_index != poll.activeSlide) gotoSlide(slide_index); gridView = false;"
                                ":style"="calculateSlideStyle(slide_index, poll.activeSlide, gridView)"
                            {
                                div x-show="gridView" x-cloak
                                    ."absolute size-full top-6 right-8 z-30 flex items-start justify-end gap-6"
                                {
                                    button "@click"="$event.stopPropagation();"
                                        ."size-28 p-5 rounded-full text-slate-400 bg-slate-50 hover:bg-slate-100 shadow-2xl"
                                        { (SvgIcon::Move.render()) }
                                    button "@click"="poll.slides.splice(slide_index, 1); if(poll.activeSlide == poll.slides.length) poll.activeSlide -= 1; $event.stopPropagation();"
                                        ."size-28 p-5 rounded-full text-slate-400 bg-slate-50 hover:bg-slate-100 shadow-2xl"
                                        { (SvgIcon::Trash2.render()) }
                                }
                                input type="text" x-model="slide.question"
                                    "@input"="save"
                                    "@keyup.enter"="questionInputEnterEvent(slide_index, slide)"
                                    ":id"="'question-input-' + slide_index"
                                    ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                    placeholder="Question"
                                    ":disabled"="isLive"
                                    ."w-full mb-3 px-1 py-0.5 text-xl text-slate-800";
                                template x-if="slide.type == 'undefined'" {
                                    div {
                                        ."mb-2 text-slate-500 tracking-tight text-center" {
                                            "Choose item type:"
                                        }
                                        ."flex justify-center gap-4" {
                                            button "@click"="slide.type = 'mc'; slide.mcAnswers.push({ text: '', isCorrect: false }, { text: '', isCorrect: false }); save(); document.getElementById('question-input-' + slide_index).focus();" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                                ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                            {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) }
                                                "Multiple choice"
                                            }
                                            button "@click"="slide.type = 'ft'; save(); document.getElementById('question-input-' + slide_index).focus();" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                                ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                            {
                                                ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                                                "Free text"
                                            }
                                        }
                                    }
                                }
                                /*template x-if="slide.type == 'firstSlide'" {
                                    div ."h-full flex justify-center items-center gap-20" {
                                        div ."p-4" {
                                            div ."mb-1 text-sm text-slate-500 text-center" {
                                                "Enter on "
                                                a x-show="code !== null" ."text-indigo-500 underline" ":href"="'/p?c=' + code" { "svoote.com" }
                                                span x-show="code === null" ."text-indigo-300 underline" { "svoote.com" }
                                            }
                                            div x-text="code !== null ? code : '0000'" ":class"="isLive ? 'text-slate-700' : 'text-slate-300'" ."mb-6 text-3xl text-center tracking-wider font-bold" {}
                                            div ."relative w-32 flex justify-center" {
                                                div #"qrcode" ":class"="isLive || 'blur-sm'" x-effect="renderQRCode($el, code)" {}
                                                div x-show="!isLive" ."absolute size-full inset-0 flex justify-center items-center" { }
                                            }
                                        }
                                        div ."w-[25rem]" {
                                            (Illustrations::TeamCollaboration.render())
                                        }
                                    }
                                }
                                template x-if="slide.type == 'lastSlide'" {
                                    div ."h-full flex flex-col justify-center" {
                                        ."mx-auto w-24" { (Illustrations::InLove.render()) }
                                        ."mt-8 text-slate-500 text-center text-sm" { "This poll has no more items. Thank you for using svoote.com" }
                                    }
                                }*/
                                template x-if="slide.type == 'mc'" {
                                    div ."relative h-full" {
                                        template x-for="(answer, answer_index) in slide.mcAnswers" {
                                            div ."mb-1.5 flex items-center gap-2" {
                                                div x-text="incrementChar('A', answer_index)" ."ml-2 text-sm text-slate-400" {}
                                                input type="text" x-model="answer.text" "@input"="save()"
                                                    "@keyup.enter"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[1].focus(); else next.click();"
                                                    ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                                    ":id"="(answer_index == 0) && 's-' + slide_index + '-mc-answer-0'"
                                                    ":disabled"="isLive"
                                                    ."w-full px-1 py-0.5 text-slate-700";
                                                button x-show="!isLive" "@click"="answer.isCorrect = !answer.isCorrect; save()" ":class"="answer.isCorrect ? 'text-green-600' : 'text-slate-300 hover:text-green-600'" ."size-6" { (SvgIcon::CheckSquare.render()) }
                                                button x-show="!isLive" "@click"="slide.mcAnswers.splice(answer_index, 1); save();" ."size-6 text-slate-300 hover:text-slate-500" { (SvgIcon::Trash2.render()) }
                                            }
                                        }
                                        button
                                            "@click"={"if (slide.mcAnswers.length < " (POLL_MAX_MC_ANSWERS) ") { slide.mcAnswers.push({ text: '', isCorrect: false }); save(); $nextTick(() => $el.previousSibling.children[1].focus()); }" }
                                            ":class"={ "(slide.mcAnswers.length >= " (POLL_MAX_MC_ANSWERS) ") && 'hidden'" }
                                            ."ml-6 text-slate-700 underline"
                                            ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                            ":id"="'add-mc-answer-' + slide_index"
                                            x-show="!isLive"
                                        {
                                            "Add answer"
                                        }
                                        div ."absolute w-full left-0 bottom-0 flex items-start justify-center gap-4" {
                                            template x-for="(answer, answer_index) in slide.mcAnswers" {
                                                div ."w-28" {
                                                    div ."h-40 flex flex-col justify-end items-center" {
                                                        div ":class"="colorPalette[answer_index % colorPalette.length]"
                                                            ":style"="`height: ${ Math.max(2, slide.stats !== null ? slide.stats.percentages[answer_index] : 2) }%;`"
                                                            ."w-16 transition-all duration-300 relative shadow-lg"
                                                        {
                                                            div x-text="`${ slide.stats !== null ? slide.stats.counts[answer_index] : 0 }`"
                                                                ."absolute w-full text-slate-600 text-center font-medium -translate-y-7" {}
                                                        }
                                                    }
                                                    div x-text="answer.text" ."h-12 mt-3 text-slate-600 text-sm text-center break-words" {}
                                                }
                                            }
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
                                                        ."block w-fit min-w-16 px-3 py-0.5 bg-slate-100 text-slate-500 rounded-full outline-none"
                                                        ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                                        ":id"="(answer_index == 0) && 's-' + slide_index + '-ft-answer-0'"
                                                        "@keydown.enter.prevent"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[0].focus(); else next.click();" {}
                                                    button "@click"="slide.ftAnswers.splice(answer_index, 1); save();" ."size-4 text-slate-300" { (SvgIcon::X.render()) }
                                                }
                                            }
                                            button "@click"="slide.ftAnswers.push({ text: '' }); save(); $nextTick(() => $el.previousSibling.children[0].focus());"
                                                ":id"="'add-ft-answer-' + slide_index"
                                                ."size-7 p-0.5 text-slate-300 border rounded-full"
                                            { (SvgIcon::Plus.render()) }
                                        }
                                        div ."text-slate-500 text-center text-sm"  { "If the leaderboard is enabled, Participants can receive points for submitting the correct answer." }
                                    }
                                }
                                div x-show="isLive" x-cloak ."absolute right-8 top-10 flex flex-col items-center" {
                                    div x-data="qrCode" x-effect="render($el, code)" ."mb-4 w-24" {}
                                    div x-text="code !== null ? code : ''" ."text-2xl text-slate-600 tracking-wide font-bold" {}
                                    div ."text-xs text-slate-500 text-center" {
                                        "Go to "
                                        a x-show="code !== null" ."text-indigo-500 underline" ":href"="'/p?c=' + code" { "svoote.com" }
                                        span x-show="code === null" ."text-indigo-300 underline" { "svoote.com" }
                                    }
                                }
                                div x-show="gridView" x-cloak ."absolute size-full inset-0" {} // Stops elements from being clicked or focused during grid view
                            }
                        }
                    }
                }
            }
        },
    );

    return Ok((cookies, html).into_response());
}

pub async fn post_start_poll(cookies: CookieJar, body: String) -> Result<Response, AppError> {
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let poll = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let enable_leaderboard = poll["enableLeaderboard"].as_bool().unwrap_or(false);
    let allow_custom_names = poll["allowCustomNames"].as_bool().unwrap_or(false);

    let (poll_id, _live_poll) = match LIVE_POLL_STORE.get_by_session_id(&session_id) {
        Some((poll_id, live_poll)) => (poll_id, live_poll),
        None => {
            let mut slides = Vec::new();

            for item in poll["slides"].as_array().ok_or(AppError::BadRequest(
                "Poll needs to contain a 'slides' array".to_string(),
            ))? {
                match item["type"].as_str().ok_or(AppError::BadRequest(
                    "type field needs to be a string".to_string(),
                ))? {
                    "firstSlide" => {
                        slides.push(Slide {
                            question: String::new(),
                            slide_type: SlideType::EntrySlide,
                            player_scores: Vec::new(),
                        });
                    }
                    "lastSlide" => {
                        slides.push(Slide {
                            question: String::new(),
                            slide_type: SlideType::FinalSlide,
                            player_scores: Vec::new(),
                        });
                    }
                    "mc" => {
                        let answers: Vec<(String, bool)> = item["mcAnswers"]
                            .as_array()
                            .ok_or(AppError::BadRequest(
                                "mcAnswers must be an array".to_string(),
                            ))?
                            .into_iter()
                            .map(|mc_answer| {
                                (
                                    mc_answer["text"].as_str().unwrap_or_default().to_string(),
                                    mc_answer["isCorrect"].as_bool().unwrap_or(false),
                                )
                            })
                            .collect();

                        slides.push(Slide {
                            question: item["question"]
                                .as_str()
                                .ok_or(AppError::BadRequest(
                                    "Question field missing for slide".to_string(),
                                ))?
                                .to_string(),
                            slide_type: SlideType::MultipleChoice(MultipleChoiceLiveAnswers {
                                answer_counts: std::iter::repeat(0usize)
                                    .take(answers.len())
                                    .collect(),
                                answers: answers.clone(),
                                player_answers: Vec::new(),
                            }),
                            player_scores: Vec::new(),
                        });
                    }
                    "ft" => {
                        let answers: Vec<SmartString<Compact>> = item["ftAnswers"]
                            .as_array()
                            .ok_or(AppError::BadRequest(
                                "mcAnswers must be an array".to_string(),
                            ))?
                            .into_iter()
                            .map(|ft_answer| {
                                SmartString::from(ft_answer["text"].as_str().unwrap_or_default())
                            })
                            .collect();

                        slides.push(Slide {
                            question: item["question"]
                                .as_str()
                                .ok_or(AppError::BadRequest(
                                    "Question field missing for slide".to_string(),
                                ))?
                                .to_string(),
                            slide_type: SlideType::FreeText(FreeTextLiveAnswers {
                                correct_answers: answers,
                                player_answers: Vec::new(),
                            }),
                            player_scores: Vec::new(),
                        });
                    }
                    _ => slides.push(Slide {
                        question: String::new(),
                        slide_type: SlideType::Undefined,
                        player_scores: Vec::new(),
                    }),
                }
            }

            let (poll_id, live_poll) =
                LivePoll::orchestrate(slides, session_id, enable_leaderboard, allow_custom_names)?;

            live_poll
                .lock()
                .unwrap()
                .start_poll_channel_sender
                .take()
                .map(|signal| {
                    let _ = signal.send(());
                });

            (poll_id, live_poll)
        }
    };

    return Ok(poll_id.to_string().into_response());
}

pub async fn post_stop_poll(
    cookies: CookieJar,
    Path(poll_id): Path<ShortID>,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let exit_channel = live_poll.lock().unwrap().exit_poll_channel_sender.clone();
    let _ = exit_channel.send(()).await;

    return Ok("Exited successfully".into_response());
}

pub async fn host_socket(
    ws: WebSocketUpgrade,
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    return Ok(ws.on_upgrade(|socket| handle_host_socket(socket, live_poll)));
}

#[derive(Deserialize, Serialize)]
struct WSMessage {
    pub cmd: SmartString<Compact>,
    pub data: Value,
}

impl Into<Message> for WSMessage {
    fn into(self) -> Message {
        return Message::Text(serde_json::to_string(&self).unwrap());
    }
}

impl WSMessage {
    fn parse(message: Message) -> Option<Self> {
        if let Ok(text) = message.into_text() {
            if let Ok(msg) = serde_json::from_str::<WSMessage>(&text) {
                return Some(msg);
            }
        }

        return None;
    }
}

async fn handle_host_socket(mut socket: WebSocket, live_poll: Arc<Mutex<LivePoll>>) {
    let (mut stats_updated_receiver, slide_index_sender) = {
        let live_poll = live_poll.lock().unwrap();

        (
            live_poll
                .stats_change_notification_channel_receiver
                .resubscribe(),
            live_poll.set_slide_index_channel_sender.clone(),
        )
    };

    loop {
        select! {
            msg = socket.recv() => {
                if let Some(Ok(msg)) = msg {
                    if let Some(msg) = WSMessage::parse(msg) {
                        match msg.cmd.as_ref() {
                            "gotoSlide" => {
                                let slide_index = msg.data["slideIndex"].as_u64().unwrap_or(0u64) as usize;
                                let _ = slide_index_sender.send(slide_index).await;
                            }
                            _ => {}
                        }
                    }
                } else {
                    return;
                }
            }
            slide_index = stats_updated_receiver.recv() => {
                if let Ok(slide_index) = slide_index {
                    let stats = match &live_poll.lock().unwrap().get_current_slide().slide_type {
                        SlideType::MultipleChoice(answers) => {
                            let max = *answers.answer_counts.iter().max().unwrap_or(&1usize);
                            let percentages: Vec<f32> = answers.answer_counts.iter()
                                .map(|count| (*count as f32 / max as f32 * 100f32).max(2f32))
                                .collect();

                            json!({
                                "counts": answers.answer_counts,
                                "percentages": percentages,
                            })
                        }
                        _ => Value::Null
                    };

                    let _  = socket.send(WSMessage {
                        cmd: SmartString::from("updateStats"),
                        data: json!({
                            "slideIndex": slide_index,
                            "stats": stats,
                        })
                    }.into()).await;
                } else {
                    return;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(15)) => {
                if socket.send(Message::Ping(Vec::new())).await.is_err() {
                    return;
                }
            }
        };
    }
}
