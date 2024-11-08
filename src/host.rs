use std::convert::Infallible;

use axum::{
    extract::Path,
    response::{sse, IntoResponse, Response, Sse},
};

use axum_extra::extract::CookieJar;
use futures::Stream;
use maud::{html, Markup};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt as _;

use crate::{
    app_error::AppError,
    config::{COLOR_PALETTE, POLL_MAX_MC_ANSWERS, POLL_MAX_SLIDES},
    html_page::{self, render_header},
    illustrations::Illustrations,
    live_poll::{LivePoll, QuestionAreaState, QuestionStatisticsState},
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    session_id,
    slide::Slide,
    static_file,
    svg_icons::SvgIcon,
};

pub async fn get_poll_page(cookies: CookieJar) -> Result<Response, AppError> {
    let (_session_id, cookies) = session_id::get_or_create_session_id(cookies);
    /*let auth_token = AuthToken::get_or_create(&session).await?;

    match LIVE_POLL_STORE
        .get_from_session(&session, &auth_token)
        .await?
    {
        Some((poll_id, _live_poll)) => return host::render_live_host(poll_id).await,
        None => {
            let poll = PersistablePoll::from_session(&session).await?;
            return render_edit_page(session, poll).await;
        }
    };*/

    let html = html_page::render_html_page(
        "Svoote",
        html! {
            script src=(static_file::get_path("qrcode.js")) {}
            #pollEditingArea ."mb-16" {
                (render_header(html! {}))
                div x-data="poll" {
                    div ."mb-4 grid grid-cols-3" {
                        div x-data="{ open: false }" ."relative" {
                            button "@click"="open = !open" ."size-6 text-slate-400 hover:text-slate-600" { (SvgIcon::Settings.render()) }
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
                        button "@click"="gotoSlide(poll.activeSlide - 1)" ":disabled"="poll.activeSlide == 0" ."z-10 relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700 disabled:bg-slate-300" {
                            ."absolute inset-0 size-full flex items-center justify-center" {
                                ."size-4 translate-x-[-0.05rem]" { (SvgIcon::ChevronLeft.render()) }
                            }
                        }
                        div ."flex-1 relative h-[36rem]" {
                            template x-for="(slide, slide_index) in poll.slides" {
                                div ."absolute inset-0 size-full px-16 py-10 border rounded transition duration-500 ease-out transform-gpu" ":class"="slide_index == poll.activeSlide && 'shadow-xl'"
                                    ":style"="'transform: perspective(100px) translateX(' + ((slide_index - poll.activeSlide) * 106) + '%) translateZ(' + (slide_index == poll.activeSlide ? '0' : '-10')  + 'px)'"
                                {
                                    input type="text" x-model="slide.question" "@input"="save" x-show="slide_index != 0 && slide_index != poll.slides.length - 1"
                                        "@keyup.enter"="questionInputEnterEvent(slide_index, slide)"
                                        ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                        placeholder="Question"
                                        ."w-full mb-6 text-xl text-slate-800 outline-none";
                                    template x-if="slide.type == 'undefined'" {
                                        div {
                                            ."mb-2 text-slate-500 tracking-tight text-center" {
                                                "Choose item type:"
                                            }
                                            ."flex justify-center gap-4" {
                                                button "@click"="slide.type = 'mc'; save();" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                                    ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                                {
                                                    ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) }
                                                    "Multiple choice"
                                                }
                                                button "@click"="slide.type = 'ft'; save();" ."px-3.5 py-2 flex justify-center items-center gap-2 text-slate-600 border rounded hover:bg-slate-100"
                                                    ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                                {
                                                    ."size-6 p-1 shrink-0 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                                                    "Free text"
                                                }
                                            }
                                        }
                                    }
                                    template x-if="slide.type == 'firstSlide'" {
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
                                                    div x-show="!isLive" ."absolute size-full inset-0 flex justify-center items-center" {
                                                        button "@click"="startPoll()" ."px-4 py-2 flex items-center gap-2 text-slate-100 bg-indigo-600 rounded-lg"
                                                            ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
                                                        {
                                                            "Start"
                                                            ."size-5" { (SvgIcon::Play.render()) }
                                                        }
                                                    }
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
                                    }
                                    template x-if="slide.type == 'mc'" {
                                        div {
                                            template x-for="(answer, answer_index) in slide.mcAnswers" {
                                                div ."mb-4 flex items-center gap-3" {
                                                    div x-text="incrementChar('A', answer_index)" ."text-slate-500" {}
                                                    input type="text" x-model="answer.text" "@input"="save()"
                                                        "@keyup.enter"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[1].focus(); else next.click();"
                                                        ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
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
                                                ":tabindex"="slide_index == poll.activeSlide ? '0' : '-1'"
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
                                }
                            }
                        }
                        button ":disabled"="poll.activeSlide + 1 == poll.slides.length" "@click"="gotoSlide(poll.activeSlide + 1)"
                            ."relative size-8 mt-20 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700 disabled:bg-slate-300"
                        {
                            ."absolute inset-0 size-full flex items-center justify-center" {
                                ."size-4" { (SvgIcon::ChevronRight.render()) }
                            }
                        }
                    }
                    div ."mt-4 flex justify-center gap-4" {
                        button "@click"="poll.slides.splice(poll.slides.length - 1, 0, createSlide(null)); $nextTick(() => { gotoSlide(poll.slides.length - 2) });"
                            ":disabled"={ "poll.slides.length >= " (POLL_MAX_SLIDES) }
                            ."group px-2 py-1 flex items-center gap-1 text-slate-500 border border-slate-300 rounded-full hover:bg-slate-100 disabled:text-slate-300 disabled:bg-transparent"
                        {
                            ."size-4 text-slate-500 group-disabled:text-slate-300" { (SvgIcon::Plus.render()) }
                            "Add slide"
                        }
                        button "@click"="poll.slides.splice(poll.activeSlide, 1); gotoSlide(poll.activeSlide - 1);" ":disabled"="poll.activeSlide == 0 || poll.activeSlide == poll.slides.length - 1"
                            ."group px-2 py-1 flex items-center gap-1 text-slate-500 border border-slate-300 rounded-full hover:bg-slate-100 disabled:text-slate-300 disabled:bg-transparent"
                        {
                            ."size-4 text-red-400 group-disabled:text-slate-300" { (SvgIcon::Trash2.render()) }
                            "Delete slide"
                        }
                    }
                }
            }
            script src=(static_file::get_path("alpine.js")) {}
        },
        true,
    );

    return Ok((cookies, html).into_response());
}

pub async fn post_start_poll(cookies: CookieJar, body: String) -> Result<Response, AppError> {
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);

    let poll = serde_json::from_str::<serde_json::Value>(&body)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let enable_leaderboard = poll["enableLeaderboard"].as_bool().unwrap_or(false);
    let allow_custom_names = poll["allowCustomNames"].as_bool().unwrap_or(false);

    error!("Json: {}", body);
    error!("Enable leaderboard: {}", enable_leaderboard);
    error!("Allow custom names: {}", allow_custom_names);

    let (poll_id, _live_poll) = match LIVE_POLL_STORE.get_by_session_id(&session_id) {
        Some((poll_id, live_poll)) => (poll_id, live_poll),
        None => {
            let mut slides = Vec::new();
            slides.push(Slide::create_join_slide());
            /*for item in poll.items {
                if let Some(live_item) = Slide::from_item(&item) {
                    slides.push(live_item);
                }
            }*/
            slides.push(Slide::create_final_slide());

            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            let (poll_id, live_poll) =
                LivePoll::orchestrate(slides, session_id, enable_leaderboard, allow_custom_names)?;

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

    return Ok(poll_id.to_string().into_response());
}
/*pub async fn render_live_host(poll_id: ShortID) -> Result<Response, AppError> {
    Ok(html_page::render_html_page(
        "Svoote",
        html! {
            (render_header(html! {}))
            ."flex flex-col gap-16" {
                div hx-ext="sse" sse-connect={"/sse/host_question/" (poll_id) } sse-close="close" {
                    div sse-swap="update" { (render_sse_loading_spinner()) }
                }
                div hx-ext="sse" sse-connect={"/sse/host_results/" (poll_id) } sse-close="close"  {
                    div sse-swap="update" { }
                }
            }
        },
        true,
    )
    .into_response())
}*/

pub async fn get_sse_slides(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let updates = live_poll.lock().unwrap().ch_question_state.clone();

    let stream = WatchStream::new(updates)
        .filter_map(move |state| match state {
            QuestionAreaState::Empty => Some(
                sse::Event::default()
                    .event("update")
                    .data(html! {}.into_string()),
            ),
            QuestionAreaState::Slide(slide_index) => {
                let current_participant_count =
                    live_poll.lock().unwrap().get_current_participant_count();
                Some(
                    sse::Event::default().event("update").data(
                        live_poll.lock().unwrap().slides[slide_index]
                            .render_host_view(poll_id, slide_index, current_participant_count)
                            .into_string(),
                    ),
                )
            }
            QuestionAreaState::PollFinished => None,
            QuestionAreaState::CloseSSE => Some(sse::Event::default().event("close").data("")),
        })
        .map(Ok);

    return Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()));
}

pub async fn get_sse_statistics(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let ch_question_statistics = live_poll
        .lock()
        .unwrap()
        .ch_question_statistics_recv
        .clone();

    let stream = WatchStream::new(ch_question_statistics)
        .map(move |statistics| match statistics {
            QuestionStatisticsState::Empty => sse::Event::default().event("update").data(""),
            QuestionStatisticsState::Slide(slide_index) => {
                sse::Event::default().event("update").data(
                    live_poll.lock().unwrap().slides[slide_index]
                        .render_statistics()
                        .into_string(),
                )
            }
            QuestionStatisticsState::CloseSSE => sse::Event::default().event("close").data(""),
        })
        .map(Ok);

    return Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()));
}

pub async fn get_sse_leaderboard(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let update_channel = live_poll.lock().unwrap().ch_players_updated_recv.clone();

    let stream = WatchStream::new(update_channel)
        .map(move |_| {
            let live_poll = live_poll.lock().unwrap();

            html! {
                ."mt-10 mb-4 flex gap-2 items-center" {
                    ."size-5 text-amber-500" { (SvgIcon::Crown.render()) }
                    ."text-xl font-medium text-slate-600" { "Leaderboard" }
                }
                ."mb-2 text-sm text-slate-600 tracking-tight" {
                    (live_poll.players.len())
                    " player" (if live_poll.players.len() != 1 { "s" } else { "" })
                }
                ."max-h-96 overflow-scroll" {
                    @for (_player_index, player) in live_poll.players.iter().enumerate() {
                        ."flex justify-between" {
                            ."flex items-center text-slate-900 gap-1" {
                                ."size-5" { (player.get_avatar_svg()) }
                                (player.get_name())
                            }
                            ."text-slate-600 font-medium tracking-tight" { (0) }
                        }
                    }
                }
            }
        })
        .map(|html| {
            sse::Event::default()
                .event("update")
                .data(html.into_string())
        })
        .map(Ok);

    Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()))
}

pub async fn post_next_slide(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let next_question_send = live_poll.lock().unwrap().ch_next_question.clone();
    let _ = next_question_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub async fn post_previous_slide(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let previous_question_send = live_poll.lock().unwrap().ch_previous_question.clone();
    let _ = previous_question_send.send(()).await;

    Ok(html! {
        p { "Success" }
    }
    .into_response())
}

pub async fn post_exit_poll(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Response, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let exit_poll_send = live_poll.lock().unwrap().ch_exit_poll.clone();
    let _ = exit_poll_send.send(()).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    return Ok("Poll exited".into_response());
}

pub fn render_sse_loading_spinner() -> Markup {
    html! {
        ."h-64 flex items-center justify-center" {
            ."size-4" { (SvgIcon::Spinner.render()) }
        }
    }
}

pub async fn get_sse_user_counter(
    Path(poll_id): Path<ShortID>,
    cookies: CookieJar,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, AppError> {
    let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;
    let (session_id, _cookies) = session_id::get_or_create_session_id(cookies);
    session_id::assert_equal_ids(&session_id, &live_poll.lock().unwrap().host_session_id)?;

    let update_channel = live_poll.lock().unwrap().ch_players_updated_recv.clone();

    let mut last_send_player_count = None;

    let stream = WatchStream::new(update_channel)
        .filter_map(move |_| {
            let live_poll = live_poll.lock().unwrap();

            if last_send_player_count.is_some_and(|count| count == live_poll.players.len()) {
                None
            } else {
                last_send_player_count = Some(live_poll.players.len());
                Some(html! { ."text-slate-600 text-lg" { (live_poll.players.len()) } })
            }
        })
        .map(|html| {
            sse::Event::default()
                .event("update")
                .data(html.into_string())
        })
        .map(Ok);

    return Ok(Sse::new(stream).keep_alive(sse::KeepAlive::default()));
}
