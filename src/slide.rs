use arrayvec::ArrayVec;
use maud::{html, Markup, PreEscaped};
use qrcode::{render::svg, QrCode};
use smartstring::{Compact, SmartString};

use crate::{
    app_error::AppError,
    config::{COLOR_PALETTE, MAX_FREE_TEXT_ANSWERS},
    illustrations::Illustrations,
    live_poll::Item,
    live_poll_store::ShortID,
    play::render_poll_finished,
    svg_icons::SvgIcon,
    word_cloud::WordCloud,
};

pub struct Slide {
    pub question: String,
    pub slide_type: SlideType,
    pub player_scores: Vec<usize>,
}

pub enum SlideType {
    EntrySlide,
    SingleChoice(MultipleChoiceLiveAnswers),
    FreeText(FreeTextLiveAnswers),
    FinalSlide,
}

pub struct MultipleChoiceLiveAnswers {
    pub answers: Vec<(String, bool)>,
    pub answer_counts: Vec<usize>,
    pub player_answers: Vec<Option<usize>>,
}

pub struct FreeTextLiveAnswers {
    pub word_cloud: WordCloud,
    pub player_answers: Vec<ArrayVec<SmartString<Compact>, MAX_FREE_TEXT_ANSWERS>>,
}

impl Slide {
    pub fn from_item(item: Item) -> Option<Slide> {
        let answers = match item.answers {
            crate::live_poll::Answers::SingleChoice(answers) => {
                SlideType::SingleChoice(MultipleChoiceLiveAnswers {
                    answer_counts: std::iter::repeat(0usize).take(answers.len()).collect(),
                    answers,
                    player_answers: Vec::new(),
                })
            }
            crate::live_poll::Answers::FreeText(_, _) => SlideType::FreeText(FreeTextLiveAnswers {
                word_cloud: WordCloud::new(),
                player_answers: Vec::new(),
            }),
            crate::live_poll::Answers::Untyped => {
                return None;
            }
        };

        return Some(Slide {
            question: item.question,
            slide_type: answers,
            player_scores: Vec::new(),
        });
    }

    pub fn create_join_slide() -> Slide {
        return Slide {
            question: String::new(),
            slide_type: SlideType::EntrySlide,
            player_scores: Vec::new(),
        };
    }

    pub fn create_final_slide() -> Slide {
        return Slide {
            question: String::new(),
            slide_type: SlideType::FinalSlide,
            player_scores: Vec::new(),
        };
    }

    pub fn is_entry_slide(&self) -> bool {
        return matches!(self.slide_type, SlideType::EntrySlide);
    }

    pub fn is_final_slide(&self) -> bool {
        return matches!(self.slide_type, SlideType::FinalSlide);
    }

    pub fn add_player(&mut self) {
        self.player_scores.push(0usize);

        match &mut self.slide_type {
            SlideType::EntrySlide => {}
            SlideType::FinalSlide => {}
            SlideType::SingleChoice(mc_answers) => {
                mc_answers.player_answers.push(None);
            }
            SlideType::FreeText(ft_answer) => {
                ft_answer.player_answers.push(ArrayVec::new());
            }
        }
    }

    pub fn render_host_view(
        &self,
        poll_id: ShortID,
        slide_index: usize,
        current_participant_count: usize,
    ) -> Markup {
        return html! {
            ."mb-6 grid grid-cols-3 items-center" {
                div {}
                ."text-center text-sm text-slate-500" {
                    @if !(self.is_entry_slide() || self.is_final_slide()) {
                        "Item " (slide_index)
                    }
                }
                ."justify-self-end" {
                    ."px-4 flex items-center gap-2 border rounded-full w-fit" {
                    ."text-slate-600 size-5 translate-y-[0.05rem]" { (SvgIcon::Users.render()) }
                        div hx-ext="sse" sse-connect={"/sse/participant_counter/" (poll_id) } sse-close="close"  {
                            div sse-swap="update" {
                                ."text-slate-600 text-lg" { (current_participant_count) }
                            }
                        }
                    }
                }
            }
            ."flex justify-between gap-8" {
                ."mt-20" {
                    button
                        hx-post={ "/previous_slide/" (poll_id) }
                        hx-swap="none"
                        ."relative group size-8 p-2 text-slate-50 rounded-full bg-slate-500 hover:bg-slate-700 disabled:opacity-0"
                        disabled[self.is_entry_slide()]
                    {
                        ."absolute inset-0 size-full flex items-center justify-center" {
                            ."size-4 translate-x-[-0.05rem]" { (SvgIcon::ChevronLeft.render()) }
                        }
                    }
                }
                ."flex-1" {
                    @if self.is_entry_slide() {
                        @let domain = "https://svoote.com";
                        @let path = format!("/p?c={}", poll_id);
                        @let complete_url = format!("{}{}", domain, path);

                        @let join_qr_code_svg = QrCode::new(&complete_url)
                            .map(|qr|
                                qr.render()
                                .min_dimensions(160, 160)
                                .quiet_zone(false)
                                .dark_color(svg::Color("#1e293b"))
                                .light_color(svg::Color("#FFFFFF"))
                                .build()
                            );

                        ."flex justify-center gap-20" {
                            ."" {
                                ."mb-1 text-sm text-slate-500 text-center" {
                                    "Enter on " a ."text-indigo-500 underline" href=(path) { "svoote.com" }
                                }
                                ."mb-6 text-3xl tracking-wider font-bold text-slate-700 text-center" {
                                    (poll_id)
                                }
                                ."w-lg flex justify-center" {
                                    (PreEscaped(join_qr_code_svg.unwrap_or("Error generating QR-Code.".to_string())))
                                }
                            }
                            ."w-[25rem]" {
                               (Illustrations::TeamCollaboration.render())
                            }
                        }
                    } @else if self.is_final_slide() {
                        ."mx-auto mt-6 w-24" { (Illustrations::InLove.render()) }
                        ."mt-8 text-slate-500 text-center text-sm" { "This poll has no more items. Thank you for using svoote.com" }
                    } @else {
                        ."mb-6 text-left text-xl text-slate-900 font-medium" { (self.question )}
                        @match &self.slide_type {
                            SlideType::EntrySlide => {} // This can never happen actually
                            SlideType::FinalSlide => {} // This can never happen actually
                            SlideType::SingleChoice(mc_answers) => {
                                @for (answer_txt, _is_correct) in &mc_answers.answers {
                                    ."p-2 mb-4 text-center text-slate-700 font-medium rounded-lg ring-2 ring-slate-500" {
                                        (answer_txt)
                                    }
                                }
                            },
                            SlideType::FreeText(_answers) => {
                                ."pl-2 flex gap-2 items-center text-slate-500" {
                                    ."size-4" { (SvgIcon::Edit3.render()) }
                                    "Submit your answer now."
                                }
                            }
                        }
                    }
                }
                ."mt-20" {
                    button
                        hx-post={ "/next_slide/" (poll_id) }
                        hx-swap="none"
                        ."relative group size-8 p-2 text-slate-50 rounded-full bg-cyan-600 hover:bg-cyan-800 disabled:opacity-0"
                        disabled[self.is_final_slide()]

                    {
                        ."absolute inset-0 size-full flex items-center justify-center" {
                            ."size-4" { (SvgIcon::ChevronRight.render()) }
                        }
                    }
                }
            }
        };
    }

    pub fn render_statistics(&mut self) -> Markup {
        match &mut self.slide_type {
            SlideType::EntrySlide => return html! {},
            SlideType::FinalSlide => return html! {},
            SlideType::SingleChoice(mc_answers) => {
                let mut max: usize = *mc_answers.answer_counts.iter().max().unwrap_or(&1usize);

                if max == 0 {
                    max = 1;
                }

                return html! {
                    ."flex flex-wrap items-start justify-center gap-6 md:gap-8 gap-y-16" {
                        @for (i, (count, (answer_txt, _is_correct))) in mc_answers.answer_counts.iter().zip(&mc_answers.answers).enumerate() {
                            ."min-w-16 md:min-w-24 max-w-36 flex-1" {
                                ."h-48 flex flex-col justify-end items-center" {
                                    #{ "ans_" (i) } // needed for the CSS height transitions
                                        ."w-24 transition-all duration-300 relative shadow-lg"
                                        .(COLOR_PALETTE[i % COLOR_PALETTE.len()])
                                        style={ "height:" (((*count as f32 / max as f32) * 100f32).max(2f32)) "%;" }
                                    {
                                        ."absolute w-full text-slate-600 text-center font-medium -translate-y-7" { (count) }
                                    }
                                }
                                ."mt-3 text-slate-600 text-sm text-center break-words" { (answer_txt) }
                            }
                        }
                    }
                };
            }
            SlideType::FreeText(ft_answers) => {
                let (words, container_height) = ft_answers.word_cloud.render();

                let html = html! {
                    @if words.len() == 0 {
                        ."mt-8 text-sm text-center text-slate-500" {
                            "The submitted answers will appear here."
                        }
                    }
                    ."relative"
                        style={ "height: " (container_height) "rem;"}
                    {
                        @for word in words {
                            ."absolute word-cloud-object text-nowrap w-full text-center overflow-hidden"
                            style={
                                "--top-rem-end:" (word.top_rem) "rem;"
                                "--top-rem-start:" (word.previous_top_rem.unwrap_or(container_height)) "rem;"

                                "--font-size-end:" (word.font_size_rem) "rem;"
                                "--font-size-start:" (word.previous_font_size_rem.unwrap_or(0.5f32)) "rem;"

                                "--text-color-end:" (word.color_code) ";"
                                "--text-color-start:" (word.previous_color_code.unwrap_or("#000000")) ";"
                            } {
                                (word.text)
                            }
                        }
                    }
                };

                ft_answers.word_cloud.save_previous();

                return html;
            }
        }
    }

    pub fn render_participant_view(
        &self,
        poll_id: ShortID,
        slide_index: usize,
        player_index: usize,
    ) -> Markup {
        return html! {
            @if self.is_entry_slide() {
                ."mx-auto mt-12 mb-6 w-44 md:w-60" { (Illustrations::People.render()) }
                ."mb-4 mx-auto size-4" { (SvgIcon::Spinner.render()) }
                ."text-center text-sm text-slate-500" {
                    "Others are joining." br;
                    "Waiting for the host to start the poll."
                }
            } @else if self.is_final_slide() {
                (render_poll_finished())
            } @else {
                ."mb-2 flex gap-6 text-sm text-slate-500" {
                    "Question " (slide_index)
                    @match &self.slide_type {
                        SlideType::EntrySlide => {}, // This can't actually happen
                        SlideType::FinalSlide => {}, // This can't actually happen
                        SlideType::SingleChoice(_mc_answers) => {
                            ."flex gap-1 items-center" {
                                ."size-4" { (SvgIcon::CheckSquare.render()) }
                                "Multiple choice"
                            }
                        }
                        SlideType::FreeText(_ft_answers) => {
                            ."flex gap-1 items-center" {
                                ."size-4" { (SvgIcon::Edit3.render()) }
                                "Free text - up to " (MAX_FREE_TEXT_ANSWERS) " answers"
                            }
                        }
                    }
                }
                ."mb-4 text-lg text-slate-700" {
                    (self.question)
                }
                @match &self.slide_type {
                    SlideType::EntrySlide => {}, // This can't actually happen
                    SlideType::FinalSlide => {}, // This can't actually happen
                    SlideType::SingleChoice(mc_answers) => {
                        @let current_mc_answer = &mc_answers.player_answers[player_index];
                        form ."block w-full" {
                            @for (answer_idx, (answer_txt, _is_correct)) in mc_answers.answers.iter().enumerate() {
                                label
                                    onclick="let e = document.getElementById('submit-btn'); if (e !== null) e.disabled = false;"
                                    .{
                                        "block p-2 mb-4 text-center text-base text-slate-700 "
                                        "rounded-lg ring-2 ring-slate-500 "
                                        "hover:ring-indigo-500 "
                                        "has-[:checked]:ring-4 has-[:checked]:ring-indigo-500 "
                                        "cursor-pointer transition duration-100 "
                                    } {
                                    (answer_txt)
                                    input ."hidden" type="radio" name="answer_idx" value=(answer_idx)
                                        required
                                        disabled[current_mc_answer.is_some()]
                                        checked[current_mc_answer.is_some_and(|ans| ans == answer_idx)];
                                }
                            }
                            ."flex justify-center mt-6" {
                                @if current_mc_answer.is_none() {
                                    button #"submit-btn"
                                        hx-post={ "/submit_mc_answer/" (poll_id) }
                                        hx-target="this"
                                        hx-swap="outerHTML"
                                        disabled
                                        ."relative group px-4 py-2 text-slate-100 tracking-wide font-semibold bg-slate-700 rounded-md hover:bg-slate-800 transition"
                                    {
                                        ."group-[.htmx-request]:opacity-0" { "Submit answer" }
                                        ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                                            ."size-4" { (SvgIcon::Spinner.render()) }
                                        }
                                    }
                                } @else {
                                    ."text-slate-700" { "Your answer has been submitted." }
                                }
                            }
                        }
                    },
                    SlideType::FreeText(ft_answers) => {
                        (ft_answers.render_form(player_index, poll_id))
                    }
                }
            }
        };
    }

    pub fn submit_score(&mut self, player_index: usize, score: usize) {
        self.player_scores[player_index] = score;
    }
}

impl MultipleChoiceLiveAnswers {
    // Returns true if the player scored points (then the leaderboard has to be updated)
    pub fn submit_answer(
        &mut self,
        player_index: usize,
        answer_index: usize,
        start_time: tokio::time::Instant,
    ) -> Result<usize, AppError> {
        if answer_index >= self.answers.len() {
            return Err(AppError::BadRequest(
                "Answer index out of bounds".to_string(),
            ));
        }

        if self.player_answers[player_index].is_some() {
            return Err(AppError::BadRequest(
                "Already submitted an answers".to_string(),
            ));
        }

        self.player_answers[player_index] = Some(answer_index);
        self.answer_counts[answer_index] += 1;

        // if answer is correct
        if self.answers[answer_index].1 {
            let mut elapsed = tokio::time::Instant::now() - start_time;
            if elapsed > tokio::time::Duration::from_secs(60) {
                elapsed = tokio::time::Duration::from_secs(60);
            }

            let fraction_points = (60_000 - elapsed.as_millis()) as f32 / 60_000f32;
            return Ok(50usize + (fraction_points * 50f32) as usize);
        }

        return Ok(0usize);
    }
}

impl FreeTextLiveAnswers {
    pub fn submit_answer(
        &mut self,
        player_index: usize,
        answer: SmartString<Compact>,
    ) -> Result<(), AppError> {
        if self.player_answers[player_index].len() >= MAX_FREE_TEXT_ANSWERS {
            return Err(AppError::BadRequest(
                "Already submitted the maximum number of free text answers ({})".to_string(),
            ));
        }

        self.word_cloud.insert(&answer);
        self.player_answers[player_index].push(answer);

        return Ok(());
    }

    pub fn render_form(&self, player_index: usize, poll_id: ShortID) -> Markup {
        let answers = &self.player_answers[player_index];

        return html! {
            form #free-text-form ."flex flex-col items-center" {
                ."w-full mb-2" {
                    @for (i, answer) in answers.iter().enumerate() {
                        ."mb-2 text-lg text-slate-700" {
                            (i + 1) ". " (answer)
                        }
                    }
                }
                @if answers.len() < MAX_FREE_TEXT_ANSWERS {
                    input type="text" name="free_text_answer" autofocus
                        ."w-full mb-4 text-lg px-2 py-1 border-2 border-slate-500 rounded-lg outline-none hover:border-indigo-600 focus:border-indigo-600 transition"
                        placeholder="Answer";
                    button
                        hx-post={ "/submit_free_text_answer/" (poll_id) }
                        hx-target="#free-text-form"
                        hx-swap="outerHTML"
                        ."relative group px-4 py-2 text-slate-100 tracking-wide font-semibold bg-slate-700 rounded-md hover:bg-slate-800 transition"
                    {
                        ."group-[.htmx-request]:opacity-0" { "Submit answer" }
                        ."absolute inset-0 size-full hidden group-[.htmx-request]:flex items-center justify-center" {
                            ."size-4" { (SvgIcon::Spinner.render()) }
                        }
                    }
                }
            }
        };
    }
}
