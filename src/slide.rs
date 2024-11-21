use arrayvec::ArrayVec;
use smartstring::{Compact, SmartString};

use crate::{app_error::AppError, config::MAX_FREE_TEXT_ANSWERS};

pub struct Slide {
    pub question: String,
    pub slide_type: SlideType,
    pub player_scores: Vec<usize>,
}

pub enum SlideType {
    Undefined,
    MultipleChoice(MultipleChoiceLiveAnswers),
    FreeText(FreeTextLiveAnswers),
    EntrySlide,
    FinalSlide,
}

pub struct MultipleChoiceLiveAnswers {
    pub answers: Vec<(String, bool)>,
    pub answer_counts: Vec<usize>,
    pub player_answers: Vec<Option<usize>>,
}

pub struct FreeTextLiveAnswers {
    pub correct_answers: Vec<SmartString<Compact>>,
    pub player_answers: Vec<ArrayVec<SmartString<Compact>, MAX_FREE_TEXT_ANSWERS>>,
}

impl Slide {
    pub fn add_player(&mut self) {
        self.player_scores.push(0usize);

        match &mut self.slide_type {
            SlideType::Undefined => {}
            SlideType::EntrySlide => {}
            SlideType::FinalSlide => {}
            SlideType::MultipleChoice(mc_answers) => {
                mc_answers.player_answers.push(None);
            }
            SlideType::FreeText(ft_answer) => {
                ft_answer.player_answers.push(ArrayVec::new());
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

        self.player_answers[player_index].push(answer);

        return Ok(());
    }

    /*pub fn render_participant_form(&self, player_index: usize, poll_id: ShortID) -> Markup {
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
    }*/
}
