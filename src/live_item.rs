use arrayvec::ArrayVec;
use maud::{html, Markup};

use crate::{
    live_poll::Item, play::MAX_FREE_TEXT_ANSWERS, svg_icons::SvgIcon, word_cloud::WordCloud,
};

pub const COLOR_PALETTE: &[&'static str] = &[
    "bg-rose-500",
    "bg-cyan-600",
    "bg-lime-500",
    "bg-fuchsia-600",
    "bg-slate-600",
    "bg-teal-500",
];

pub struct LiveItem {
    pub question: String,
    pub answers: LiveAnswers,
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
    pub player_answers: Vec<ArrayVec<String, MAX_FREE_TEXT_ANSWERS>>,
}

impl LiveItem {
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
}

impl From<Item> for LiveItem {
    fn from(item: Item) -> Self {
        let answers = match item.answers {
            crate::live_poll::Answers::SingleChoice(answers) => {
                LiveAnswers::SingleChoice(MultipleChoiceLiveAnswers {
                    answer_counts: std::iter::repeat(0usize).take(answers.len()).collect(),
                    answers,
                    player_answers: Vec::new(),
                })
            }
            crate::live_poll::Answers::FreeText(_) => LiveAnswers::FreeText(FreeTextLiveAnswers {
                word_cloud: WordCloud::new(),
                player_answers: Vec::new(),
            }),
        };

        return LiveItem {
            question: item.question,
            answers,
        };
    }
}
