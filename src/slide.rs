use std::collections::HashMap;

use arrayvec::ArrayVec;
use serde::Serialize;
use smartstring::{Compact, SmartString};

use crate::{app_error::AppError, config::POLL_MAX_MC_ANSWERS};

pub struct Slide {
    pub question: String,
    pub slide_type: SlideType,
    pub player_scores: Vec<usize>,
}

pub enum SlideType {
    Undefined,
    MultipleChoice(MultipleChoiceLiveAnswers),
    FreeText(FreeTextLiveAnswers),
}

pub struct MultipleChoiceLiveAnswers {
    pub answers: Vec<(String, bool)>,
    pub answer_counts: Vec<usize>,
    pub player_answers: Vec<Option<ArrayVec<u8, POLL_MAX_MC_ANSWERS>>>,
    pub allow_multiple_answers: bool,
}

pub struct FreeTextLiveAnswers {
    //pub correct_answers: Vec<SmartString<Compact>>,
    pub player_answers: Vec<Option<SmartString<Compact>>>,
    pub word_cloud_terms: Vec<WordCloudTerm>,
    pub max_term_count: usize,
}

#[derive(Serialize)]
pub struct WordCloudTerm {
    pub lowercase_text: SmartString<Compact>,
    pub count: usize,
    pub preferred_spelling: SmartString<Compact>,
    pub spellings: HashMap<SmartString<Compact>, usize>,
    pub highest_spelling_count: usize,
}

impl Slide {
    pub fn add_player(&mut self) {
        self.player_scores.push(0usize);

        match &mut self.slide_type {
            SlideType::Undefined => {}
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
    // Returns the scored points
    pub fn submit_answer(
        &mut self,
        player_index: usize,
        answer_indices: ArrayVec<u8, POLL_MAX_MC_ANSWERS>,
        _start_time: tokio::time::Instant,
    ) -> Result<usize, AppError> {
        if answer_indices.len() == 0 {
            return Err(AppError::BadRequest(
                "Can't submit an empty answer_indices array".to_string(),
            ));
        }

        if !self.allow_multiple_answers && answer_indices.len() > 1 {
            return Err(AppError::BadRequest(
                "Can't submit more than one answer to this MC question".to_string(),
            ));
        }

        for answer_index in &answer_indices {
            if *answer_index as usize >= self.answers.len() {
                return Err(AppError::BadRequest(
                    "Answer index out of bounds".to_string(),
                ));
            }
        }

        if self.player_answers[player_index].is_some() {
            return Err(AppError::BadRequest(
                "Already submitted an answers".to_string(),
            ));
        }

        for answer_index in &answer_indices {
            self.answer_counts[*answer_index as usize] += 1;
            // if answer is correct
            /*if self.answers[answer_index].1 {
                let mut elapsed = tokio::time::Instant::now() - start_time;
                if elapsed > tokio::time::Duration::from_secs(60) {
                    elapsed = tokio::time::Duration::from_secs(60);
                }

                let fraction_points = (60_000 - elapsed.as_millis()) as f32 / 60_000f32;
                return Ok(50usize + (fraction_points * 50f32) as usize);
            }*/
        }

        self.player_answers[player_index] = Some(answer_indices);

        return Ok(0usize);
    }
}
