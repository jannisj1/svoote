use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    http::HeaderMap,
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
    select_language, session_id,
    slide::{FreeTextLiveAnswers, MultipleChoiceLiveAnswers, Slide, SlideType, WordCloudTerm},
    static_file,
    svg_icons::SvgIcon,
    wsmessage::WSMessage,
};

pub async fn get_host_page(cookies: CookieJar, headers: HeaderMap) -> Result<Response, AppError> {
    let l = select_language(&cookies, &headers);
    let (session_id, cookies) = session_id::get_or_create_session_id(cookies);
    let poll_is_live = LIVE_POLL_STORE.get_by_session_id(&session_id).is_some();

    let html = html_page::render_html_page(
        "Svoote - Create Poll",
        &l,
        html! {
            script src=(static_file::get_path("qrcode.js")) {}
            @if poll_is_live { script { "document.pollAlreadyLive = true;" } }
            div x-data="poll" ."flex-1 flex flex-col" {
                div ."block px-6 sm:px-14 py-5 flex justify-end bg-slate-900" {
                    button #"start-stop-button"
                        "@click"="if (!isLive) startPoll(); else stopPoll();" ":disabled"="poll.slides.length == 0"
                        ."px-4.5 py-2 flex items-center justify-end gap-1.5 text-sm text-slate-700 font-medium rounded-full cursor-pointer transition-all duration-[200ms] disabled:pointer-events-none hover:shadow-none disabled:shadow-none"
                        ":class"="isLive ? 'bg-red-500 hover:bg-red-700' : 'bg-white hover:bg-slate-200 disabled:bg-slate-500'"
                        ":title"={ "!isLive ? '" (t!("start_poll_btn_title", locale=l)) "' : '" (t!("stop_poll_btn_title", locale=l)) "'" }
                    {
                        span { (t!("start_poll_btn_title", locale=l)) }
                        div x-show="!isLive" ."size-5 translate-x-0.5 translate-y-[0.05rem]"
                            { (SvgIcon::Play.render()) }
                        div x-show="isLive" ."size-5 flex items-center justify-center"
                            { ."size-3 bg-slate-50" {} }
                    }
                }
                div ."hidden [@media_(max-width:520px)]:block mx-6 mb-4 px-4 py-3 text-sm bg-orange-100 rounded-lg text-slate-500" { (t!("screen_size_warning", locale=l)) }
                div id="fullscreen-container"
                    "@fullscreenchange"="if (document.fullscreenElement == null) isFullscreen = false; else isFullscreen = true; $dispatch('fontsizechange');"
                    ":class"="'min-w-[520px] flex-1 px-6 sm:px-14 flex flex-col ' + (isFullscreen ? 'bg-slate-700 h-full justify-center' : 'bg-slate-100')"
                    x-data="{ slide: null }"
                    x-init="slide = poll.slides[poll.activeSlide];"
                    x-effect="slide = poll.slides[poll.activeSlide]"
                {
                    div ."flex-1 my-8 flex justify-center gap-6" {
                        div { // Left slide navigation sidebar
                            div ."px-1 mt-3 flex justify-between" {
                                h1 ."text-lg text-slate-500 font-medium leading-5" { (t!("slides", locale=l)) }
                                div x-data="{ open: false }" ."relative size-4" {
                                    button ."size-full text-slate-500 disabled:text-slate-300 cursor-pointer disabled:cursor-default"
                                        "@click"="open = !open"
                                        ":disabled"="isLive"
                                        title=(t!("settings_btn_title", locale=l))
                                        { (SvgIcon::Settings.render()) }
                                    div x-show="open" x-cloak
                                        "@click.outside"="open = false"
                                        ."absolute left-0 top-6 w-40 z-20 px-3 py-2 bg-white border rounded-lg shadow-lg"
                                    {
                                        button ."flex gap-2 items-center text-sm text-red-500 cursor-pointer disabled:cursor-default disabled:text-slate-300"
                                            "@click"="poll.slides.splice(poll.activeSlide, 1); open = false; gotoSlide(poll.activeSlide);"
                                            ":disabled"="isLive || poll.slides.length < 2"
                                        {
                                            ."size-4 shrink-0" { (SvgIcon::Trash2.render()) }
                                            (t!("delete_slide_btn_title", locale=l))
                                        }
                                    }
                                }
                            }
                            button ."mb-5 px-1 text-xs text-cyan-600 underline cursor-pointer"
                                "@click"="poll.slides.push(createSlide('mc')); $nextTick(() => { gotoSlide(poll.slides.length - 1) });"
                                { (t!("add_slide_btn_title", locale=l)) }
                            div ."h-[32em] p-1 flex flex-col gap-4 overflow-y-scroll scrollbar-hidden" {
                                template x-for="(s, i) in poll.slides" {
                                    div ."w-24 h-13.5 shrink-0 flex justify-center items-center text-sm font-bold bg-white rounded-lg cursor-pointer hover:bg-slate-50"
                                        x-text="i + 1" "@click"="gotoSlide(i)"
                                        ":class"="i == poll.activeSlide ? 'ring-2 ring-cyan-600' : 'ring ring-slate-200'" {}
                                }
                            }
                        }
                        div ."w-[64em] h-[36em] shrink-0 px-[3em] py-[2.5em] flex gap-[3.5em] bg-white border rounded-lg"
                            ":style"="`font-size: ${fontScale}em;`"
                        {
                            div ."w-full flex-1 flex flex-col" {
                                template x-if="slide.type == 'mc'" {
                                    div ."relative h-full flex flex-col gap-[1.5em] justify-between" {
                                        div ."flex gap-[1em]" {
                                            div ."flex-1" {
                                                div ."absolute pointer-events-none px-[0.55em] text-[1.25em] text-slate-300" x-show="slide.question.trim() === ''" { (t!("question_placeholder", locale=l)) }
                                                span x-init="$el.innerText = slide.question"
                                                    "@input"="slide.question = $el.innerText; save();"
                                                    ":contenteditable"="!isLive"
                                                    ."block mb-3 px-[0.5em] text-[1.25em] text-slate-800 bg-transparent outline-hidden"
                                                    ":class"="!isLive && 'ring-1 ring-slate-200 ring-offset-4 rounded-xs focus:ring-2 focus:ring-cyan-600'" { }
                                                template x-for="(answer, answer_index) in slide.mcAnswers" {
                                                    div ."mb-[0.375em] flex items-center gap-[0.5em]" {
                                                        div x-text="incrementChar('A', answer_index)" ."ml-[0.5em] text-[0.875em] text-slate-400" {}
                                                        input type="text" x-model="answer.text" "@input"="save(); if (slide.mcChartType == 'pie') renderPieChart();"
                                                            "@keydown.enter"="let next = $el.parentElement.nextSibling; if (next.tagName == 'DIV') next.children[1].focus(); else next.click();"
                                                            ":disabled"="isLive"
                                                            ."w-full px-[0.25em] py-[0.125em] text-slate-700 bg-transparent outline-hidden"
                                                            ":class"="!isLive && 'focus:ring-2 ring-cyan-600 ring-offset-2 rounded-xs'";
                                                        //button x-show="!isLive" "@click"="answer.isCorrect = !answer.isCorrect; save()" ":class"="answer.isCorrect ? 'text-green-600' : 'text-slate-300 hover:text-green-600'" ."size-6" { (SvgIcon::CheckSquare.render()) }
                                                        button x-show="!isLive" "@click"="slide.mcAnswers.splice(answer_index, 1); save(); $nextTick(() => { if (slide.mcChartType == 'pie') renderPieChart(); });" ."size-[1.5em] text-slate-300 cursor-pointer hover:text-slate-500" { (SvgIcon::Trash2.render()) }
                                                    }
                                                }
                                                button
                                                    "@click"={"slide.mcAnswers.push({ text: '', isCorrect: false }); save(); $nextTick(() => $el.previousSibling.children[1].focus()); if (slide.mcChartType == 'pie') renderPieChart();" }
                                                    ":class"={ "(slide.mcAnswers.length >= " (POLL_MAX_MC_ANSWERS) ") && 'hidden'" }
                                                    ."ml-[1.5em] text-slate-700 underline cursor-pointer"
                                                    x-show="!isLive"
                                                    { (t!("add_answer_btn", locale=l)) }
                                            }
                                            div x-show="isLive" x-cloak ."sm:translate-x-[1.5em] -translate-y-[1em] flex flex-col items-center" {
                                                div x-data="qrCode" x-effect="render($el, code)" ."mb-[0.75em] w-[4em] sm:w-[6em]" {}
                                                div x-text="code !== null ? '#' + code : ''" ."text-[1.25em] text-slate-600 tracking-wide font-bold" {}
                                                a x-show="code !== null" ."text-center text-[0.75em] text-indigo-500 underline" ":href"="'/p?c=' + code" { "svoote.com" }
                                            }
                                        }
                                        template x-if="slide.mcChartType == 'bar'" {
                                            div ."flex-1 max-h-[10em] flex items-start justify-center gap-[1em]" {
                                                template x-for="(answer, answer_index) in slide.mcAnswers" {
                                                    div ."h-full w-[7em]" {
                                                        div ."relative h-[calc(100%-2.5em)] flex flex-col justify-end items-center" {
                                                            div ":class"="colorPalette[answer_index % colorPalette.length]"
                                                                ":style"="`height: ${ Math.max(2, slide.stats !== null ? slide.stats.percentages[answer_index] : 2) }%;`"
                                                                ."absolute w-[4em] transition-all duration-400 shadow-lg"
                                                            {
                                                                div x-text="`${ slide.stats !== null ? slide.stats.counts[answer_index] : 0 }`"
                                                                    ."absolute w-full text-slate-600 text-center font-medium -translate-y-[1.75em]" {}
                                                            }
                                                        }
                                                        div x-text={ "answer.text != '' ? answer.text : '" (t!("answer", locale=l)) " ' + incrementChar('A', answer_index)" }
                                                            ."h-[3.25em] my-[0.5em] text-[0.875em] text-center break-words overflow-hidden"
                                                            ":class"="answer.text != '' ? 'text-slate-600' : 'text-slate-400'"
                                                        {}
                                                    }
                                                }
                                            }
                                        }
                                        template x-if="slide.mcChartType == 'pie'" {
                                            div ."flex-1 min-h-[8em] max-h-[16em]" {
                                                canvas ."size-full" #"pie-chart-canvas"
                                                    x-init="$nextTick(() => { renderPieChart() });"
                                                    "@resize.window"="$nextTick(() => { renderPieChart(); })"
                                                    "@fontsizechange.window"="$nextTick(() => { renderPieChart(); });"
                                                    "@slidechange.window"="if (isLive && slide.stats == null && poll.activeSlide != poll.priorActiveSlide) { clearPieChart(); } else setTimeout(() => { renderPieChart(); }, 500);"
                                                {}
                                            }
                                        }
                                    }
                                }
                                template x-if="slide.type == 'ft'" {
                                    div ."h-full flex flex-col" {
                                        div ."flex gap-[1em]" {
                                            div ."flex-1" {
                                                div ."absolute pointer-events-none px-[0.55em] text-[1.25em] text-slate-300" x-show="slide.question.trim() === ''" { (t!("question_placeholder", locale=l)) }
                                                span x-init="$el.innerText = slide.question"
                                                    "@slidechange.window"="if () $nextTick(() => { $el.innerText = poll.slides[poll.activeSlide].question; console.log($el.innerText); });"
                                                    "@input"="slide.question = $el.innerText; save();"
                                                    ":contenteditable"="!isLive"
                                                    ."block mb-[0.75em] px-[0.5em] text-[1.25em] text-slate-800 bg-transparent outline-hidden"
                                                    ":class"="!isLive && 'ring-1 ring-slate-200 ring-offset-4 rounded-xs focus:ring-2 focus:ring-cyan-600'" {}
                                            }
                                            div x-show="isLive" x-cloak ."sm:translate-x-[1.5em] -translate-y-[1em] flex flex-col items-center" {
                                                div x-data="qrCode" x-effect="render($el, code)" ."mb-[0.75em] w-[6em]" {}
                                                div x-text="code !== null ? '#' + code : ''" ."text-[1.25em] text-slate-600 tracking-wide font-bold" {}
                                                a x-show="code !== null" ."text-center text-[0.75em] text-indigo-500 underline" ":href"="'/p?c=' + code" { "svoote.com" }
                                            }
                                        }
                                        div ."relative flex-1 mx-auto w-full" #"word-cloud"
                                            "@resize.window"="$nextTick(() => { renderWordCloud(); })"
                                            "@fontsizechange.window"="setTimeout(() => { renderWordCloud(); }, 500);"
                                            "@slidechange.window"="setTimeout(() => { renderWordCloud(); }, 500);"
                                            { }
                                        div x-show="(slide.stats !== null ? slide.stats.terms : []).length == 0"
                                            ."absolute size-full inset-0 -z-10 p-[3em] flex items-center justify-center gap-[0.75em] text-slate-500 text-[0.875em]"
                                            { div ."size-[1em]" { (SvgIcon::Edit3.render()) } (t!("open_ended_explanation", locale=l)) }
                                    }
                                }
                                p x-show="poll.slides.length == 0" x-cloak ."absolute inset-0 px-6 size-full flex justify-center items-center text-slate-500 text-[0.875em]"
                                    { (t!("no_slides_notice", locale=l)) }
                            }
                        }
                        div ."w-72 self-stretch px-4 py-3 bg-white border rounded-lg"
                        {
                            h2 ."mb-2 px-3 text-sm text-slate-500 font-medium" { (t!("choose_template_heading", locale=l)) }
                            div ."px-3 flex flex-col gap-2" {
                                button
                                    "@click"="slide.type = 'mc'; save();"
                                    ."w-full px-2 py-1.5 flex items-center gap-2 text-slate-500 text-sm rounded ring-cyan-600 transition-all duration-100 cursor-pointer hover:bg-slate-100"
                                    ":class"="slide.type == 'mc' && 'ring-2'"
                                {
                                    div ."size-4 p-0.5 text-slate-100 rounded-xs"
                                        ":class"={ "slide.type == 'mc' ? '" (COLOR_PALETTE[0]) "' : 'bg-slate-400'" }
                                        { (SvgIcon::BarChart2.render()) }
                                    "Multiple Choice"
                                }
                                button
                                    "@click"="slide.type = 'ft'; save();"
                                    ."w-full px-2 py-1.5 flex items-center gap-2 text-slate-500 text-sm rounded ring-cyan-600 transition-all duration-100 cursor-pointer hover:bg-slate-100"
                                    ":class"="slide.type == 'ft' && 'ring-2'"
                                {
                                    //div ."size-4 p-0.5 text-slate-100 rounded-xs" .(COLOR_PALETTE[1]) { (SvgIcon::Edit3.render()) }
                                    div ."size-4 p-0.5 text-slate-100 rounded-xs"
                                        ":class"={ "slide.type == 'ft' ? '" (COLOR_PALETTE[1]) "' : 'bg-slate-400'" }
                                        { (SvgIcon::Edit3.render()) }
                                    (t!("open_ended_question", locale=l))
                                }
                            }
                            hr ."mx-3 my-5";
                            div x-show="slide.type == 'mc'" {
                                h2 ."mb-4 px-3 text-sm text-slate-500 font-medium" { (t!("choose_mc_chart_type", locale=l)) }
                                div ."mb-8 px-3 flex gap-2 text-slate-500 text-xs" {
                                    button ."px-2 py-1.5 flex-1 flex flex-col items-center gap-1.5 ring-cyan-600 rounded cursor-pointer hover:bg-slate-100"
                                        ":class"="slide.mcChartType == 'bar' && 'ring-2 pointer-events-none'"
                                        "@click"="slide.mcChartType = 'bar'; save();" {
                                            ."size-5" { (SvgIcon::BarChart2.render()) }
                                            p { (t!("bar_chart", locale=l)) }
                                    }
                                    button ."px-2 py-1.5 flex-1 flex flex-col items-center gap-1.5 ring-cyan-600 rounded cursor-pointer hover:bg-slate-100 "
                                        ":class"="slide.mcChartType == 'pie' && 'ring-2 pointer-events-none'"
                                        "@click"="slide.mcChartType = 'pie'; save(); setTimeout(() => { renderPieChart(); }, 50);" {
                                            ."size-5" { (SvgIcon::PieChart.render()) }
                                            p { (t!("pie_chart", locale=l)) }
                                    }
                                }
                                h2 ."mb-3 px-3 text-sm text-slate-500 font-medium" { (t!("other_options", locale=l)) }
                                label ."mx-5 flex gap-3 items-center text-sm text-slate-500" {
                                    input x-model="slide.allowMultipleMCAnswers" "@change"="save()" type="checkbox" ."accent-cyan-600";
                                    (t!("allow_multiple_answers", locale=l))
                                }
                            }
                        }
                    }
                    //@if cfg!(debug_assertions) { button "@click"="runDemo()" { "Run demo" } }
                    /*div ."relative mx-6 sm:mx-16 flex justify-between items-center" ":class"="isFullscreen && 'mt-6'"
                    {
                        div ."pr-4 flex items-center gap-1.5 z-10 transition" ":class"="isFullscreen && 'opacity-0'" {
                            div x-data="{ open: false }" ."relative size-[1.4rem]" {
                                button "@click"="open = !open"
                                    ":disabled"="isLive" ."size-[1.4rem] cursor-pointer disabled:cursor-default"
                                    ":class"="isFullscreen ? 'disabled:text-slate-500' : 'disabled:text-slate-300'"
                                    title=(t!("settings_btn_title", locale=l))
                                    { (SvgIcon::Settings.render()) }
                                div x-show="open" x-cloak "@click.outside"="open = false" ."absolute left-0 top-8 w-72 h-fit z-20 px-4 py-3 bg-white border rounded-lg shadow-lg" {
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
                                    label ."mb-3 flex gap-2 items-center text-slate-600 cursor-pointer hover:bg-slate-100" {
                                        ."size-4" { (SvgIcon::Folder.render()) } (t!("import_presentation", locale=l))
                                        input type="file" accept=".json" "@change"="importJsonFile($event);" ."hidden";
                                    }
                                    a download="poll.json" ":href"="'data:application/json;charset=utf-8,' + JSON.stringify(poll)"
                                        ."mb-3 flex gap-2 items-center text-slate-600 hover:bg-slate-100"
                                        { ."size-4" { (SvgIcon::Download.render()) } (t!("download_copy", locale=l)) }
                                    hr ."mb-3";
                                    button "@click"="reset()" ":disabled"="isLive" ."flex gap-2 items-center text-slate-600 cursor-pointer disabled:cursor-default disabled:text-slate-300" {
                                        ."size-4" { (SvgIcon::Refresh.render()) }
                                        (t!("reset_btn_text", locale=l))
                                    }
                                }
                            }
                            button "@click"="poll.slides.splice(poll.activeSlide, 1); gotoSlide(poll.activeSlide);"
                                ":disabled"="isLive"
                                ."size-6 cursor-pointer disabled:cursor-default"
                                ":class"="isFullscreen ? 'disabled:text-slate-500' : 'disabled:text-slate-300'"
                                title=(t!("delete_slide_btn_title", locale=l))
                                { (SvgIcon::Trash2.render()) }
                            button "@click"="poll.slides.splice(poll.slides.length, 0, createSlide('mc')); $nextTick(() => { gotoSlide(poll.slides.length - 1) });"
                                ":disabled"={ "isLive || poll.slides.length >= " (POLL_MAX_SLIDES) }
                                ."-translate-x-1 size-6 cursor-pointer disabled:cursor-default"
                                ":class"="isFullscreen ? 'disabled:text-slate-500' : 'disabled:text-slate-300'"
                                title=(t!("add_slide_btn_title", locale=l)) { (SvgIcon::Plus.render()) }
                        }
                        div ."pl-4 flex items-center gap-3 z-10" ":class"="isFullscreen ? 'bg-slate-700' : 'bg-white'" {
                            div x-show="isFullscreen" x-cloak x-effect="$dispatch('fontsizechange', { size: fontSize })"
                                ."mr-2 flex items-baseline gap-2 text-slate-500 font-mono font-bold"
                            {
                                label ."text-sm cursor-pointer has-checked:text-slate-100" title=(t!("text_size_medium", locale=l)) { "Aa" input type="radio" ."hidden" x-model="fontSize" name="fontSize" value="medium"; }
                                label ."text-large cursor-pointer has-checked:text-slate-100" title=(t!("text_size_large", locale=l)) { "Aa" input type="radio" ."hidden" x-model="fontSize" name="fontSize" value="large"; }
                                label ."text-xl cursor-pointer has-checked:text-slate-100" title=(t!("text_size_xlarge", locale=l)) { "Aa" input type="radio" ."hidden" x-model="fontSize" name="fontSize" value="xlarge"; }
                            }
                            button "@click"="toggleFullscreen()" ":disabled"="!isLive" x-show="document.documentElement.requestFullscreen != null"
                                ."p-2 bg-white border rounded-full shadow-xs hover:bg-slate-200 cursor-pointer disabled:cursor-default hover:shadow-none disabled:shadow-none disabled:text-slate-300 disabled:bg-white"
                                title=(t!("fullscreen_btn_title", locale=l))
                            {
                                template x-if="!isFullscreen" { div ."size-5" { (SvgIcon::Maximize.render()) } }
                                template x-if="isFullscreen" { div ."size-5" { (SvgIcon::Minimize.render()) } }
                            }
                        }
                    }*/
                    div ."h-12 mx-6 sm:mx-14 mt-2 mb-8 grid grid-cols-3 items-center gap-4" { // The fixed height stops ugly re-layout when a reaction smiley is first sent
                        div { }
                        div ."flex justify-center items-center gap-5" {
                            button ."p-2 size-8 rounded-full shadow-xs cursor-pointer hover:shadow-none disabled:pointer-events-none disabled:text-slate-400"
                                ":class"="isFullscreen ? 'bg-slate-300 hover:bg-slate-100' : 'bg-slate-100 hover:bg-slate-200'"
                                "@click"="gotoSlide(poll.activeSlide - 1)"
                                ":disabled"="poll.activeSlide == 0"
                                title=(t!("prev_slide_btn", locale=l))
                                { (SvgIcon::ArrowLeft.render()) }
                            div x-text={ "'" (t!("slide", locale=l)) " ' + (poll.activeSlide + 1)" } ."text-sm" ":class"="isFullscreen ? 'text-slate-300' : 'text-slate-500'" {}
                            button ."p-2 size-8 rounded-full shadow-xs cursor-pointer hover:shadow-none disabled:pointer-events-none disabled:text-slate-400"
                                ":class"="isFullscreen ? 'bg-slate-300 hover:bg-slate-100' : 'bg-slate-100 hover:bg-slate-200'"
                                "@click"="gotoSlide(poll.activeSlide + 1)"
                                ":disabled"="poll.activeSlide == poll.slides.length - 1"
                                title=(t!("next_slide_btn", locale=l))
                                { (SvgIcon::ArrowRight.render()) }
                        }
                        div {
                            template x-if="isLive && poll.slides[poll.activeSlide].emojis" {
                                div ."flex justify-end items-center gap-2"
                                    ":class"="isFullscreen && (fontSize == 'large' ? 'text-[1.4rem]' : (fontSize == 'xlarge' ? 'text-[1.8rem]' : 'text-base'))" {
                                        div id="emoji-counter-heart" x-show="poll.slides[poll.activeSlide].emojis.heart > 0" ."relative px-[0.5em] py-[0.25em] text-[0.75em] border rounded-full" ":class"="isFullscreen ? 'border-slate-500 text-slate-300' : 'text-slate-500'" {
                                            "❤️ "  span x-text="poll.slides[poll.activeSlide].emojis.heart" {}
                                        }
                                        div id="emoji-counter-thumbsUp" x-show="poll.slides[poll.activeSlide].emojis.thumbsUp > 0" ."relative px-[0.5em] py-[0.25em] text-[0.75em] border rounded-full" ":class"="isFullscreen ? 'border-slate-500 text-slate-300' : 'text-slate-500'" {
                                            "👍 "  span x-text="poll.slides[poll.activeSlide].emojis.thumbsUp" {}
                                        }
                                        div id="emoji-counter-thumbsDown" x-show="poll.slides[poll.activeSlide].emojis.thumbsDown > 0" ."relative px-[0.5em] py-[0.25em] text-[0.75em] border rounded-full" ":class"="isFullscreen ? 'border-slate-500 text-slate-300' : 'text-slate-500'" {
                                            "👎 "  span x-text="poll.slides[poll.activeSlide].emojis.thumbsDown" {}
                                        }
                                        div id="emoji-counter-smileyFace" x-show="poll.slides[poll.activeSlide].emojis.smileyFace > 0" ."relative px-[0.5em] py-[0.25em] text-[0.75em] border rounded-full" ":class"="isFullscreen ? 'border-slate-500 text-slate-300' : 'text-slate-500'" {
                                            "😀 "  span x-text="poll.slides[poll.activeSlide].emojis.smileyFace" {}
                                        }
                                        div id="emoji-counter-sadFace" x-show="poll.slides[poll.activeSlide].emojis.sadFace > 0" ."relative px-[0.5em] py-[0.25em] text-[0.75em] border rounded-full" ":class"="isFullscreen ? 'border-slate-500 text-slate-300' : 'text-slate-500'" {
                                            "🙁 "  span x-text="poll.slides[poll.activeSlide].emojis.sadFace" {}
                                        }
                                }
                            }
                        }
                    }
                }
            }
        },
        false,
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
                            player_emojis: Vec::new(),
                            heart_emojis: 0,
                            thumbs_up_emojis: 0,
                            thumbs_down_emojis: 0,
                            smiley_face_emojis: 0,
                            sad_face_emojis: 0,
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
                            player_emojis: Vec::new(),
                            heart_emojis: 0,
                            thumbs_up_emojis: 0,
                            thumbs_down_emojis: 0,
                            smiley_face_emojis: 0,
                            sad_face_emojis: 0,
                        });
                    }
                    _ => slides.push(Slide {
                        question: String::new(),
                        slide_type: SlideType::Undefined,
                        player_scores: Vec::new(),
                        player_emojis: Vec::new(),
                        heart_emojis: 0,
                        thumbs_up_emojis: 0,
                        thumbs_down_emojis: 0,
                        smiley_face_emojis: 0,
                        sad_face_emojis: 0,
                    }),
                }
            }

            if slides.len() == 0 {
                slides.push(Slide {
                    question: String::new(),
                    slide_type: SlideType::Undefined,
                    player_scores: Vec::new(),
                    player_emojis: Vec::new(),
                    heart_emojis: 0,
                    thumbs_up_emojis: 0,
                    thumbs_down_emojis: 0,
                    smiley_face_emojis: 0,
                    sad_face_emojis: 0,
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
    let (
        mut stats_updated_receiver,
        mut slide_change_notification_receiver,
        mut emoji_receiver,
        slide_index_sender,
    ) = {
        let live_poll = live_poll.lock().unwrap();

        (
            live_poll
                .stats_change_notification_channel_receiver
                .resubscribe(),
            live_poll
                .slide_change_notification_channel_receiver
                .resubscribe(),
            live_poll.emoji_channel_receiver.resubscribe(),
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
                    if slide_index < live_poll.lock().unwrap().slides.len() {
                        let stats = match &live_poll.lock().unwrap().slides[slide_index].slide_type {
                            SlideType::MultipleChoice(answers) => {
                                json!({ "counts": answers.answer_counts })
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
                    }
                } else {
                    return;
                }
            }
            slide_index = slide_change_notification_receiver.recv() => {
                if let Ok(slide_index) = slide_index {
                    if slide_index < live_poll.lock().unwrap().slides.len() {
                        let msg = {
                            let slide = &live_poll.lock().unwrap().slides[slide_index];
                            WSMessage {
                                cmd: SmartString::from("setEmojiCounts"),
                                data: json!({
                                    "slideIndex": slide_index,
                                    "emojis": {
                                        "heart": slide.heart_emojis,
                                        "thumbsUp": slide.thumbs_up_emojis,
                                        "thumbsDown": slide.thumbs_down_emojis,
                                        "smileyFace": slide.smiley_face_emojis,
                                        "sadFace": slide.sad_face_emojis,
                                    },
                                })
                            }.into()
                        };

                        let _  = socket.send(msg).await;
                    }
                } else {
                    return;
                }
            }
            emoji = emoji_receiver.recv() => {
                if let Ok((slide_index, emoji)) = emoji {
                    let msg = WSMessage {
                        cmd: SmartString::from("newEmoji"),
                        data: json!({
                            "slideIndex": slide_index,
                            "emoji": emoji,
                        })
                    }.into();

                    let _  = socket.send(msg).await;
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
            "en",
            html! {
                (render_header(html!{}))
                div ."my-32 mx-auto max-w-96 p-4 text-center border rounded-lg shadow-xs" {
                    h1 ."text-xl font-bold text-slate-600" { "Svoote live statistics" }
                    p ."" { "Number of live polls: " (stats.num_live_polls) }
                    p ."" { "Number of participants: " (stats.num_participants) }
                    p ."" { "Avg. participants per poll: " (format!("{:.2}", stats.num_participants as f32 / stats.num_live_polls as f32)) }
                }
            },
            true
        )
        .into_response());
    }

    return Err(AppError::OtherInternalServerError(
        "Failure getting cached website stats".to_string(),
    ));
}
