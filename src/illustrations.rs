/*
These amazing illustrations were obtained from undraw.co.
Be sure to check them out!
*/

use maud::PreEscaped;

pub enum Illustrations {
    InLove,
    //People,
    //TeamCollaboration,
}

impl Illustrations {
    pub fn render(&self) -> PreEscaped<&'static str> {
        return PreEscaped(match self {
            Self::InLove => include_str!("static/svgs/undraw_love_it_heart.svg"),
            //Self::People => include_str!("static/svgs/undraw_people.svg"),
            //Self::TeamCollaboration => include_str!("static/svgs/undraw_team_collaboration.svg"),
        });
    }
}
