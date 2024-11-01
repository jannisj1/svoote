/*
These amazing illustrations were obtained from undraw.co.
Be sure to check them out!
*/

use maud::PreEscaped;

pub enum Illustrations {
    Quiz,
    InLove,
}

impl Illustrations {
    pub fn render(&self) -> PreEscaped<&'static str> {
        return PreEscaped(match self {
            Self::Quiz => include_str!("static/svgs/undraw_quiz.svg"),
            Self::InLove => include_str!("static/svgs/undraw_love_it_heart.svg"),
        });
    }
}
