use arrayvec::ArrayVec;
use maud::{html, Markup};
use smartstring::{Compact, SmartString};

use crate::{
    app_error::AppError,
    config::{COLOR_PALETTE, MAX_FREE_TEXT_ANSWERS},
    live_poll::Item,
    live_poll_store::ShortID,
    svg_icons::SvgIcon,
    word_cloud::WordCloud,
};

pub struct LiveItem {
    pub question: String,
    pub answers: LiveAnswers,
    pub player_scores: Vec<usize>,
}

pub enum LiveAnswers {
    SingleChoice(MultipleChoiceLiveAnswers),
    FreeText(FreeTextLiveAnswers),
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

impl LiveItem {
    pub fn new(item: Item) -> Self {
        let answers = match item.answers {
            crate::live_poll::Answers::SingleChoice(answers) => {
                LiveAnswers::SingleChoice(MultipleChoiceLiveAnswers {
                    answer_counts: std::iter::repeat(0usize).take(answers.len()).collect(),
                    answers,
                    player_answers: Vec::new(),
                })
            }
            crate::live_poll::Answers::FreeText(_, _) => {
                LiveAnswers::FreeText(FreeTextLiveAnswers {
                    word_cloud: WordCloud::new(),
                    player_answers: Vec::new(),
                })
            }
        };

        return LiveItem {
            question: item.question,
            answers,
            player_scores: Vec::new(),
        };
    }

    pub fn add_player(&mut self) {
        self.player_scores.push(0usize);

        match &mut self.answers {
            LiveAnswers::SingleChoice(mc_answers) => {
                mc_answers.player_answers.push(None);
            }
            LiveAnswers::FreeText(ft_answer) => {
                ft_answer.player_answers.push(ArrayVec::new());
            }
        }
    }

    pub fn render_host_view(&self) -> Markup {
        return html! {
            ."mb-6 text-left text-xl text-slate-900 font-medium" { (self.question )}
            @match &self.answers {
                LiveAnswers::SingleChoice(mc_answers) => {
                    @for (answer_txt, _is_correct) in &mc_answers.answers {
                        ."p-2 mb-4 text-center text-slate-700 font-medium rounded-lg ring-2 ring-slate-500" {
                            (answer_txt)
                        }
                    }
                },
                LiveAnswers::FreeText(_answers) => {
                    ."pl-2 flex gap-2 items-center text-slate-500" {
                        ."size-4" { (SvgIcon::Edit3.render()) }
                        "Submit your answer now."
                    }
                }
            }
        };
    }

    pub fn render_statistics(&mut self) -> Markup {
        match &mut self.answers {
            LiveAnswers::SingleChoice(mc_answers) => {
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
            LiveAnswers::FreeText(ft_answers) => {
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
