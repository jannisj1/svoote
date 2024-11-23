use serde::Serialize;
use smartstring::{Compact, SmartString};

use crate::app_error::AppError;

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
    pub player_answers: Vec<Option<SmartString<Compact>>>,
    pub word_cloud_terms: Vec<WordCloudTerm>,
    pub max_term_count: usize,
}

#[derive(Serialize)]
pub struct WordCloudTerm {
    pub text: SmartString<Compact>,
    pub count: usize,
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
                ft_answer.player_answers.push(None);
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
