use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::{IntoResponse, Response},
};

use axum_extra::extract::CookieJar;
use maud::html;
use serde_json::{json, Value};
use smartstring::SmartString;
use tokio::select;

use crate::{
    app_error::AppError,
    config::{COLOR_PALETTE, POLL_MAX_MC_ANSWERS, POLL_MAX_SLIDES, STATS_UPDATE_THROTTLE},
    html_page::{self, render_header},
    live_poll::LivePoll,
    live_poll_store::{ShortID, LIVE_POLL_STORE},
    session_id,
    slide::{FreeTextLiveAnswers, MultipleChoiceLiveAnswers, Slide, SlideType, WordCloudTerm},
    static_file,
    svg_icons::SvgIcon,
    wsmessage::WSMessage,
};

pub async fn get_poll_page(cookies: CookieJar) -> Result<Response, AppError> {
    let (session_id, cookies) = session_id::get_or_create_session_id(cookies);

    let poll_is_live = LIVE_POLL_STORE.get_by_session_id(&session_id).is_some();

    let html = html_page::render_html_page(
        "Svoote",
        html! {
            script src=(static_file::get_path("qrcode.js")) {}
            @if poll_is_live { script { "document.pollAlreadyLive = true;" } }
            (render_header(html! { a href="/p" ."text-slate-500 text-sm underline" { "Join presentation" } }))
            div x-data="poll" id="fullscreen-container" "@fullscreenchange"="if (document.fullscreenElement == null) isFullscreen = false; else isFullscreen = true;"
                ":class"="isFullscreen ? 'bg-slate-700 h-full flex flex-col justify-center' : 'bg-white'"
            {
                div ."relative mx-6 sm:mx-16 flex justify-between items-center"
                    ":class"="isFullscreen && 'mt-6'"
                {
                    div ."absolute size-0 left-1/2 top-1.5" {
                        template x-for="i in poll.slides.length" {
                            button x-text="i"
                                ."absolute top-0 left-1/2 size-6 rounded-full text-sm font-mono transition-all duration-500 ease-out disabled:opacity-0"
                                ":style"="`transform: translateX(${ ((i - 1) - poll.activeSlide) * 24 - 12 }px);`"
                                ":class"="i - 1 == poll.activeSlide ? 'bg-slate-500 text-slate-50' : (isFullscreen ? 'text-slate-100' : '')"
                                ":disabled"="Math.abs((i - 1) - poll.activeSlide) > 6"
                                "@click"="gotoSlide(i - 1)"
                            { }
                        }
                    }
                    div ."pr-4 flex items-center gap-2 z-10" ":class"="isFullscreen ? 'bg-slate-700' : 'bg-white'" {
                        div x-data="{ open: false }" ."relative size-[1.4rem]" {
                            button "@click"="open = !open"
                                ":disabled"="isLive" ."size-[1.4rem]"
                                ":class"="isFullscreen ? 'disabled:text-slate-500' : 'disabled:text-slate-300'"
                                title="Settings"
                                { (SvgIcon::Settings.render()) }
                            div x-show="open" x-cloak "@click.outside"="open = false" ."absolute left-0 top-8 w-64 h-fit z-20 p-4 text-left bg-white border rounded-lg shadow-lg" {
                                /*."mb-3 text-xl font-semibold text-slate-700" { "Options" }
                                label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                    input type="checkbox" x-model="poll.enableLeaderboard" ."size-4 accent-indigo-500";
                                    "Leaderboard"
                                }
                                ."ml-6 mb-3 text-slate-400 text-sm" { "Participants will receive points for submitting the correct answer. Faster responses get more points." }
                                label ."flex items-center gap-2 text-slate-600 font-semibold" {
                                    input type="checkbox" x-model="poll.allowCustomNames" ."size-4 accent-indigo-500";
                                    "Custom names"
                                }
                                ."ml-6 mb-3 text-slate-400 text-sm" { "Allow participants to set a custom name." }*/
                                a download="poll.json" ":href"="'data:application/json;charset=utf-8,' + JSON.stringify(poll)"
                                    ."mb-2 flex gap-2 items-center text-slate-600 hover:text-slate-900"
                                    { ."size-4" { (SvgIcon::Save.render()) } "Save presentation (.json)" }
                                label ."mb-3 flex gap-2 items-center text-slate-600 cursor-pointer hover:text-slate-900" {
                                    ."size-4" { (SvgIcon::Folder.render()) } "Load presentation (.json)"
                                    input type="file" accept=".json" "@change"="importJsonFile($event);" ."hidden";
                                }
                                hr ."mb-3";
                                button "@click"="reset()" ":disabled"="isLive" ."flex gap-2 items-center text-slate-600 disabled:text-slate-300" {
                                    ."size-4" { (SvgIcon::Refresh.render()) }
                                    "Reset slides and settings"
                                }
                            }
                        }
                        button "@click"="gridView = !gridView;"
                            ":disabled"="isLive" ."size-6"
                            ":class"="gridView ? 'text-indigo-500' : (isFullscreen ? 'disabled:text-slate-500' : 'disabled:text-slate-300')"
                            title="Grid view" { (SvgIcon::Grid.render()) }
                        button "@click"="poll.slides.splice(poll.slides.length, 0, createSlide('mc')); $nextTick(() => { gotoSlide(poll.slides.length - 1) });"
                            ":disabled"={ "isLive || poll.slides.length >= " (POLL_MAX_SLIDES) }
                            ."-translate-x-1 size-6"
                            ":class"="isFullscreen ? 'disabled:text-slate-500' : 'disabled:text-slate-300'"
                            title="Add new slide" { (SvgIcon::Plus.render()) }
                    }
                    div ."pl-4 flex items-center gap-3 z-10" ":class"="isFullscreen ? 'bg-slate-700' : 'bg-white'" {
                        div x-show="isFullscreen" x-cloak x-effect="$dispatch('fontsizechange', { size: fontSize })"
                            ."mr-2 flex items-baseline gap-2 text-slate-500 font-mono font-bold"
                        {
                            label ."text-sm cursor-pointer has-[:checked]:text-slate-100" title="Text size medium" { "Aa" input type="radio" ."hidden" x-model="fontSize" name="fontSize" value="medium"; }
                            label ."text-large cursor-pointer has-[:checked]:text-slate-100" title="Text size large" { "Aa" input type="radio" ."hidden" x-model="fontSize" name="fontSize" value="large"; }
                            label ."text-xl cursor-pointer has-[:checked]:text-slate-100" title="Text size extra-large" { "Aa" input type="radio" ."hidden" x-model="fontSize" name="fontSize" value="xlarge"; }
                        }
                        button x-show="!isLive" "@click"="startPoll()"
                            ":disabled"="poll.slides.length == 0"
                            ."p-2 text-slate-50 bg-green-500 rounded-full shadow shadow-slate-400 hover:bg-green-600 hover:shadow-none disabled:bg-green-200 disabled:shadow-none"
                            title="Start poll"
                            { ."size-5 translate-x-0.5 translate-y-[0.05rem]" { (SvgIcon::Play.render()) } }
                        button x-show="isLive" x-cloak "@click"="stopPoll()"
                            ."p-3 text-slate-50 bg-red-500 rounded-full hover:bg-red-700"
                            title="Stop poll"
                            { ."size-3 bg-slate-50" {} }
                        button "@click"="toggleFullscreen()" ":disabled"="!isLive"
                            ."p-2 bg-white border rounded-full shadow hover:bg-slate-200 hover:shadow-none disabled:shadow-none disabled:text-slate-300 disabled:bg-white"
                            title="Toggle fullscreen mode"
                        {
                            template x-if="!isFullscreen" { div ."size-5" { (SvgIcon::Maximize.render()) } }
                            template x-if="isFullscreen" { div ."size-5" { (SvgIcon::Minimize.render()) } }
                        }
                    }
                }
                div x-ref="outerSlideContainer" ."px-4 pt-3 pb-2 sm:px-12 overflow-x-hidden overflow-y-scroll scrollbar-hidden"
                    ":class"="isFullscreen && ('flex-1 ' + (fontSize == 'large' ? 'text-[1.25rem]' : (fontSize == 'xlarge' ? 'text-[1.5rem]' : 'text-base')))" {
                    div ."relative" ":class"="isFullscreen ? 'h-full' : 'h-[36rem]'" {
                        p x-show="poll.slides.length == 0" x-cloak ."absolute inset-0 px-6 size-full flex justify-center items-center text-slate-500 text-[0.875em]" { "Empty presentation, add slides by clicking '+' in the top left." }
                        template x-for="(slide, slideIndex) in poll.slides" {
                            div
                                ":class"="calculateSlideClasses(slideIndex, poll.activeSlide, gridView)"
                                ":style"="calculateSlideStyle(slideIndex, poll.activeSlide, gridView, isLive)"
                                "@click"="if (slideIndex != poll.activeSlide) gotoSlide(slideIndex); if (gridView) { gridView = false; $refs.outerSlideContainer.scrollTo({ top: 0, behavior: 'smooth' }); }"
                            {
                                div x-data="{ selectTemplate: false }" ."flex-1 flex flex-col" {
                                    h1 x-show="gridView" x-cloak x-text="'Slide ' + (slideIndex + 1)" ."absolute text-5xl text-slate-500 -top-20 left-[45%]" {}
                                    button "@click"="isReordering = !isReordering; reorderedSlideIndex = slideIndex; $event.stopPropagation();"
                                        x-show="!isLive && gridView && (!isReordering || slideIndex == reorderedSlideIndex)" x-cloak
                                        ."absolute top-6 right-8 size-28 p-5 z-30 rounded-full text-slate-400 bg-slate-50 hover:bg-slate-100 shadow-2xl"
                                        { (SvgIcon::Move.render()) }
                                    button "@click"="poll.slides.splice(slideIndex, 1); gotoSlide(poll.activeSlide); $event.stopPropagation();"
                                        x-show="!isLive && gridView && !isReordering" x-cloak
                                        ."absolute top-6 right-44 z-30 size-28 p-5 rounded-full text-slate-400 bg-slate-50 hover:bg-slate-100 shadow-2xl"
                                        { (SvgIcon::Trash2.render()) }
                                    button x-show="gridView && isReordering && slideIndex % 3 == 0" x-cloak ."absolute h-full w-[14%] top-0 -left-[17%] z-40 rounded-lg bg-red-200 hover:bg-red-300"
                                        "@click"="$event.stopPropagation(); moveSlide(slideIndex, true); isReordering = false;"
                                        { }
                                    button x-show="gridView && isReordering" x-cloak ."absolute h-full w-[14%] top-0 -right-[17%] z-40 rounded-lg bg-red-200 hover:bg-red-300"
                                        "@click"="$event.stopPropagation(); moveSlide(slideIndex, false); isReordering = false;"
                                        { }
                                    button x-show="!gridView && !isLive" x-cloak ."absolute top-4 right-4 size-5 text-slate-400 hover:text-red-500"
                                        "@click"="poll.slides.splice(slideIndex, 1); gotoSlide(poll.activeSlide);"
                                        title="Delete slide"
                                        { (SvgIcon::X.render()) }
                                    div ."absolute inset-0 size-full transition duration-300 z-10"
                                        ":class"="selectTemplate ? 'backdrop-blur-sm' : 'pointer-events-none'"
                                        x-show="!isLive"
                                        "@click"="selectTemplate = false" {}
                                            h2 ."absolute left-1/2 top-[1em] -translate-x-1/2 z-10 text-[0.875em] text-slate-500 transition duration-300 "
                                        ":class"="selectTemplate ? '' : 'opacity-0'"
                                        x-show="!isLive"
                                        { "Choose template:" }
                                    button
                                        "@click"="if (!selectTemplate) { selectTemplate = true; } else { selectTemplate = false; slide.type = 'mc'; } save();"
                                        ":class"="calculateSlideTypeButtonClasses(slide.type, 'mc', selectTemplate)"
                                        x-show="!isLive"
                                        { ."size-6 p-1 text-slate-100 rounded" .(COLOR_PALETTE[0]) { (SvgIcon::BarChart2.render()) } "Multiple choice" }
                                    button
                                        "@click"="if (!selectTemplate) { selectTemplate = true; } else { selectTemplate = false; slide.type = 'ft'; } save();"
                                        ":class"="calculateSlideTypeButtonClasses(slide.type, 'ft', selectTemplate)"
                                        x-show="!isLive"
                                        { ."size-6 p-1 text-slate-100 rounded" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) } "Open ended" }
                                    template x-if="slide.type == 'mc'" {
                                        div ."relative h-full flex flex-col gap-[3em] justify-between" {
                                            div ."flex gap-[1em]" {
                                                div ."flex-1" {
                                                    div ."-z-10 absolute px-[0.25em] py-[0.125em] text-[1.25em] text-slate-500 bg-transparent" x-cloak x-show="slide.question.trim() == '' && !isLive" { "Question" }
                                                    span x-init="$el.innerText = slide.question"
                                                        "@input"="slide.question = $el.innerText; save();"
                                                        ":id"="'question-input-' + slideIndex" ":tabindex"="slideIndex == poll.activeSlide ? '0' : '-1'"
                                                        ":contenteditable"="!isLive"
                                                        ."block mb-3 px-[0.25em] py-[0.125em] text-[1.25em] text-slate-800 bg-transparent" {}
                                                    label /*x-show="!isLive" x-collapse*/ ."ml-[0.5em] mb-[0.5em] overflow-hidden flex gap-[0.5em] items-center text-slate-700 transition-all duration-500"
                                                        ":class"="isLive ? 'h-0 opacity-0' : 'h-[1.5em]'" {
                                                        input x-model="slide.allowMultipleMCAnswers" "@change"="save()" type="checkbox" ."accent-indigo-500";
                                                        "Allow multiple answers per user"
                                                    }
                                                    template x-for="(answer, answer_index) in slide.mcAnswers" {
                                                        div ."mb-[0.375em] flex items-center gap-[0.5em]" {
                                                            div x-text="incrementChar('A', answer_index)" ."ml-[0.5em] text-[0.875em] text-slate-400" {}
                                                            input type="text" x-model="answer.text" "@input"="save()"
                                                                "@keydown.enter"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[1].focus(); else next.click();"
                                                                ":tabindex"="slideIndex == poll.activeSlide ? '0' : '-1'"
                                                                ":id"="(answer_index == 0) && 's-' + slideIndex + '-mc-answer-0'"
                                                                ":disabled"="isLive"
                                                                ."w-full px-[0.25em] py-[0.125em] text-slate-700 bg-transparent";
                                                            //button x-show="!isLive" "@click"="answer.isCorrect = !answer.isCorrect; save()" ":class"="answer.isCorrect ? 'text-green-600' : 'text-slate-300 hover:text-green-600'" ."size-6" { (SvgIcon::CheckSquare.render()) }
                                                            button x-show="!isLive" "@click"="slide.mcAnswers.splice(answer_index, 1); save();" ."size-[1.5em] text-slate-300 hover:text-slate-500" { (SvgIcon::Trash2.render()) }
                                                        }
                                                    }
                                                    button
                                                        "@click"={"if (slide.mcAnswers.length < " (POLL_MAX_MC_ANSWERS) ") { slide.mcAnswers.push({ text: '', isCorrect: false }); save(); $nextTick(() => $el.previousSibling.children[1].focus()); }" }
                                                        ":class"={ "(slide.mcAnswers.length >= " (POLL_MAX_MC_ANSWERS) ") && 'hidden'" }
                                                        ."ml-[1.5em] text-slate-700 underline"
                                                        ":tabindex"="slideIndex == poll.activeSlide ? '0' : '-1'"
                                                        ":id"="'add-mc-answer-' + slideIndex"
                                                        x-show="!isLive"
                                                        { "Add answer" }
                                                }
                                                div x-show="isLive" x-cloak ."sm:translate-x-[1.5em] -translate-y-[1em] flex flex-col items-center" {
                                                    div x-data="qrCode" x-effect="if (slideIndex == poll.activeSlide) render($el, code)" ."mb-[0.75em] w-[4em] sm:w-[6em]" {}
                                                    div x-text="code !== null ? code : ''" ."text-[1.25em] text-slate-600 tracking-wide font-bold" {}
                                                    a x-show="code !== null" ."text-center text-[0.75em] text-indigo-500 underline" ":href"="'/p?c=' + code" { "svoote.com" }
                                                }
                                            }
                                            div ."flex-1 max-h-[10em] flex items-start justify-center gap-[1em]" {
                                                template x-for="(answer, answer_index) in slide.mcAnswers" {
                                                    div ."h-full w-[7em]" {
                                                        div ."h-[calc(100%-2.5em)] flex flex-col justify-end items-center" {
                                                            div ":class"="colorPalette[answer_index % colorPalette.length]"
                                                                ":style"="`height: ${ Math.max(2, slide.stats !== null ? slide.stats.percentages[answer_index] : 2) }%;`"
                                                                ."w-[4em] transition-all duration-300 relative shadow-lg"
                                                            {
                                                                div x-text="`${ slide.stats !== null ? slide.stats.counts[answer_index] : 0 }`"
                                                                    ."absolute w-full text-slate-600 text-center font-medium -translate-y-[1.75em]" {}
                                                            }
                                                        }
                                                        div x-text="answer.text != '' ? answer.text : 'Answer ' + incrementChar('A', answer_index)" ."h-[2.5em] my-[0.5em] text-slate-600 text-[0.875em] text-center break-words overflow-hidden" {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    template x-if="slide.type == 'ft'" {
                                        div ."h-full flex flex-col" {
                                            div ."flex gap-[1em]" {
                                                div ."flex-1" {
                                                    div ."-z-10 absolute px-[0.25em] py-[0.125em] text-[1.25em] text-slate-500 bg-transparent" x-cloak x-show="slide.question.trim() == '' && !isLive" { "Question" }
                                                    span x-init="$el.innerText = slide.question"
                                                        "@input"="slide.question = $el.innerText; save();"
                                                        ":id"="'question-input-' + slideIndex" ":tabindex"="slideIndex == poll.activeSlide ? '0' : '-1'"
                                                        ":contenteditable"="!isLive"
                                                        ."block mb-[0.75em] px-[0.25em] py-[0.125em] text-[1.25em] text-slate-800 bg-transparent" {}
                                                }
                                                div x-show="isLive" x-cloak ."sm:translate-x-[1.5em] -translate-y-[1em] flex flex-col items-center" {
                                                    div x-data="qrCode" x-effect="if (slideIndex == poll.activeSlide) render($el, code)" ."mb-[0.75em] w-[6em]" {}
                                                    div x-text="code !== null ? code : ''" ."text-[1.25em] text-slate-600 tracking-wide font-bold" {}
                                                    a x-show="code !== null" ."text-center text-[0.75em] text-indigo-500 underline" ":href"="'/p?c=' + code" { "svoote.com" }
                                                }
                                            }
                                            div ."relative flex-1 mx-auto w-full"
                                                ":id"="`word-cloud-${slideIndex}`"
                                                "@resize.window"="$nextTick(() => { renderWordCloud(slideIndex); })"
                                                "@fontsizechange.window"="console.log('RedraRedraww'); setTimeout(() => { renderWordCloud(slideIndex); }, 500);"
                                                "@slidechange.window"="setTimeout(() => { renderWordCloud(slideIndex); }, 500);"
                                                { }
                                            div x-show="(slide.stats !== null ? slide.stats.terms : []).length == 0"
                                                ."absolute size-full inset-0 -z-10 p-[1.5em] flex items-center justify-center gap-[0.5em] text-slate-500 text-[0.875em]"
                                                { div ."size-[1em]" { (SvgIcon::Edit3.render()) } "Open-ended: Answers will show up here in a word cloud." }
                                        }
                                    }
                                }
                                div x-show="gridView" x-cloak ."absolute size-full inset-0" {} // Stops elements from being clicked or focused during grid view
                            }
                        }
                    }
                }
                div ."mt-2 mb-6 flex justify-center gap-4" {
                    button ."p-2 size-8 rounded-full shadow hover:shadow-none disabled:pointer-events-none disabled:text-slate-400"
                        ":class"="isFullscreen ? 'bg-slate-300 hover:bg-slate-100' : 'bg-slate-100 hover:bg-slate-200'"
                        "@click"="gotoSlide(poll.activeSlide - 1)"
                        ":disabled"="poll.activeSlide == 0"
                        { (SvgIcon::ArrowLeft.render()) }
                    button ."p-2 size-8 rounded-full shadow hover:shadow-none disabled:pointer-events-none disabled:text-slate-400"
                        ":class"="isFullscreen ? 'bg-slate-300 hover:bg-slate-100' : 'bg-slate-100 hover:bg-slate-200'"
                        "@click"="gotoSlide(poll.activeSlide + 1)"
                        ":disabled"="poll.activeSlide == poll.slides.length - 1"
                        { (SvgIcon::ArrowRight.render()) }
                }
            }
            p ."mb-4 text-center text-sm text-slate-500" {
                "Svoote is a new and growing open-source project. "
                "Please leave your feedback and issues on "
                a href="https://github.com/jannisj1/svoote" ."underline" { "Github" }
                "."
            }
            div ."mx-6 flex justify-center flex-wrap gap-x-6 gap-y-4 items-center" {
                button onclick="document.getElementById('help-dialog').showModal();"
                    ."px-3 py-1 flex items-center gap-1.5 text-sm text-slate-500 border rounded-full hover:bg-slate-100"
                    { "How to use Svoote" div ."size-5" { (SvgIcon::Help.render()) } }
                a href="/about" ."px-3 py-1 flex items-center gap-1.5 text-sm text-slate-500 border rounded-full hover:bg-slate-100"
                    { "About Svoote " div ."size-4" { (SvgIcon::Rss.render()) } }
            }
            dialog id="help-dialog" ."fixed inset-0" {
                div ."max-w-96 px-8 py-6 rounded-lg" {
                    form method="dialog" ."flex justify-end" { button ."size-6 text-red-500" { (SvgIcon::X.render()) } }
                    h1 ."mb-6 text-xl text-slate-500 font-semibold" { "Help" }
                    ul ."ml-6 list-disc text-slate-500 space-y-1" {
                        li { "Add slides by clicking the plus button (" div ."inline-block size-4 translate-y-[0.2rem]" { (SvgIcon::Plus.render()) } ") in the top left and fill the slides with your content." }
                        li { "To remove slides or change the order of them, go to the grid view via the grid view button (" div ."inline-block size-4 translate-y-[0.2rem]" { (SvgIcon::Grid.render()) } ") in the top left." }
                        li { "Start the interactive presentation by clicking the start button (" div ."inline-block size-4 translate-y-[0.2rem]" { (SvgIcon::Play.render()) } ") in the top right. A QR-Code will show up on the slides to let participants join your presentation." }
                        li { "When you are finished with your presentation, you can stop it by clicking on the stop button ( " div ."inline-block size-3 bg-slate-500 translate-y-[0.1rem]" {} " ) in the top right." }
                        li { "Your slides are saved locally in your browser. If you wish to transfer them to another device or store them for a longer time, click on the settings button (" div ."inline-block size-4 translate-y-[0.2rem]" { (SvgIcon::Settings.render()) } ") in the top left and then on 'Save presentation'. You can later import the slides via 'Load presentation'." }
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

    //let enable_leaderboard = poll["enableLeaderboard"].as_bool().unwrap_or(false);
    //let allow_custom_names = poll["allowCustomNames"].as_bool().unwrap_or(false);

    let (poll_id, _live_poll) = match LIVE_POLL_STORE.get_by_session_id(&session_id) {
        Some((poll_id, live_poll)) => (poll_id, live_poll),
        None => {
            let mut slides = Vec::new();

            for slide in poll["slides"].as_array().ok_or(AppError::BadRequest(
                "Poll needs to contain a 'slides' array".to_string(),
            ))? {
                match slide["type"].as_str().ok_or(AppError::BadRequest(
                    "type field needs to be a string".to_string(),
                ))? {
                    "mc" => {
                        let answers: Vec<(String, bool)> = slide["mcAnswers"]
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
                            question: slide["question"]
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
                                allow_multiple_answers: slide["allowMultipleMCAnswers"]
                                    .as_bool()
                                    .unwrap_or(false),
                            }),
                            player_scores: Vec::new(),
                        });
                    }
                    "ft" => {
                        /*let answers: Vec<SmartString<Compact>> = item["ftAnswers"]
                        .as_array()
                        .ok_or(AppError::BadRequest(
                            "mcAnswers must be an array".to_string(),
                        ))?
                        .into_iter()
                        .map(|ft_answer| {
                            SmartString::from(ft_answer["text"].as_str().unwrap_or_default())
                        })
                        .collect();*/

                        slides.push(Slide {
                            question: slide["question"]
                                .as_str()
                                .ok_or(AppError::BadRequest(
                                    "Question field missing for slide".to_string(),
                                ))?
                                .to_string(),
                            slide_type: SlideType::FreeText(FreeTextLiveAnswers {
                                //correct_answers: answers,
                                player_answers: Vec::new(),
                                word_cloud_terms: Vec::new(),
                                max_term_count: 1usize,
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

            if slides.len() == 0 {
                slides.push(Slide {
                    question: String::new(),
                    slide_type: SlideType::Undefined,
                    player_scores: Vec::new(),
                });
            }

            let (poll_id, live_poll) = LivePoll::orchestrate(slides, session_id)?;

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

    let mut last_sent_timepoint = tokio::time::Instant::now() - STATS_UPDATE_THROTTLE;
    let mut throttled_msg = None;

    loop {
        let throttled_msg_sent_timeout = if throttled_msg.is_some() {
            STATS_UPDATE_THROTTLE
                .checked_sub(tokio::time::Instant::now() - last_sent_timepoint)
                .unwrap_or(tokio::time::Duration::from_secs(0))
        } else {
            tokio::time::Duration::from_secs(999999)
        };

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
                        SlideType::FreeText(answers) => {
                            json!({
                                "terms": answers.word_cloud_terms
                                    .iter()
                                    .map(|term| (term.preferred_spelling.clone(), term.count))
                                    .collect::<Vec<_>>(),
                                "maxCount": answers.max_term_count,
                            })
                        }
                        _ => Value::Null
                    };

                    let msg = WSMessage {
                        cmd: SmartString::from("updateStats"),
                        data: json!({
                            "slideIndex": slide_index,
                            "stats": stats,
                        })
                    }.into();

                    if throttled_msg.is_none() &&
                        tokio::time::Instant::now() - last_sent_timepoint > STATS_UPDATE_THROTTLE {
                        let _  = socket.send(msg).await;
                        last_sent_timepoint = tokio::time::Instant::now();
                        throttled_msg = None;
                    } else {
                        throttled_msg = Some(msg);
                    }
                } else {
                    return;
                }
            }
            _ = tokio::time::sleep(throttled_msg_sent_timeout) => {
                if let Some(msg) = throttled_msg.take() {
                    let _  = socket.send(msg).await;
                    last_sent_timepoint = tokio::time::Instant::now();
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

pub async fn get_bombardft(Path(poll_id): Path<ShortID>) -> Result<Response, AppError> {
    if cfg!(debug_assertions) {
        let live_poll = LIVE_POLL_STORE.get(poll_id).ok_or(AppError::NotFound)?;

        tokio::spawn(async move {
            let mut i = 0;
            loop {
                i += 1;
                let _ = tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                let mut live_poll = live_poll.lock().unwrap();
                if let SlideType::FreeText(answers) = &mut live_poll.get_current_slide().slide_type
                {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();

                    answers.word_cloud_terms.push(WordCloudTerm {
                        lowercase_text: SmartString::from(i.to_string()),
                        count: 1,
                        preferred_spelling: SmartString::from(i.to_string()),
                        highest_spelling_count: 1,
                        spellings: HashMap::new(),
                    });

                    let random_int = rng.gen_range::<usize, _>(0..20);
                    if random_int < answers.word_cloud_terms.len() {
                        answers.word_cloud_terms[random_int].count += 1;
                        if answers.word_cloud_terms[random_int].count > answers.max_term_count {
                            answers.max_term_count = answers.word_cloud_terms[random_int].count;
                        }
                    }

                    let _ = live_poll
                        .stats_change_notification_channel_sender
                        .send(live_poll.current_slide_index);
                }
            }
        });

        return Ok("Starting bombarding...".into_response());
    } else {
        return Err(AppError::BadRequest(
            "Only available in debug mode.".to_string(),
        ));
    }
}

struct WebsiteStats {
    pub timepoint: tokio::time::Instant,
    pub num_live_polls: usize,
    pub num_participants: usize,
}
static STATS: Mutex<Option<WebsiteStats>> = Mutex::new(None);

pub async fn get_stats() -> Result<Response, AppError> {
    use tokio::time::{Duration, Instant};
    let mut stats = STATS.lock().unwrap();

    if stats.is_none()
        || stats
            .as_ref()
            .is_some_and(|stats| (Instant::now() - stats.timepoint) >= Duration::from_secs(5))
    {
        let polls = LIVE_POLL_STORE.polls.lock().unwrap();

        *stats = Some(WebsiteStats {
            timepoint: Instant::now(),
            num_live_polls: polls.len(),
            num_participants: polls
                .iter()
                .map(|p| p.1.lock().unwrap().players.len())
                .sum::<usize>(),
        });
    }

    if let Some(stats) = &*stats {
        return Ok(html_page::render_html_page(
            "Svoote Live Stats",
            html! {
                (render_header(html!{}))
                div ."my-32 mx-auto max-w-96 p-4 text-center border rounded-lg shadow" {
                    h1 ."text-xl font-bold text-slate-600" { "Svoote live statistics" }
                    p ."" { "Number of live polls: " (stats.num_live_polls) }
                    p ."" { "Number of participants: " (stats.num_participants) }
                    p ."" { "Avg. participants per poll: " (format!("{:.2}", stats.num_participants as f32 / stats.num_live_polls as f32)) }
                }
            },
        )
        .into_response());
    }

    return Err(AppError::OtherInternalServerError(
        "Failure getting cached website stats".to_string(),
    ));
}
