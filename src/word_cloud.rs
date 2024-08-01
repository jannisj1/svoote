static WORD_CLOUD_COLORS: &[&'static str] = &[
    "#f43f5e", // bg-rose-500
    "#0891b2", // bg-cyan-600
    "#84cc16", // bg-lime-500
    "#c026d3", // bg-fuchsia-600
    "#4b5563", // bg-slate-600
    "#14b8a6", // bg-teal-500
];

const VERTICAL_WORD_GAP_REM: f32 = 0.75f32;

pub struct WordCloudObject {
    pub text: String,
    pub count: usize,
    pub font_size_rem: f32,
    pub top_rem: f32,
    pub color_code: &'static str,
    pub previous_font_size_rem: Option<f32>,
    pub previous_top_rem: Option<f32>,
    pub previous_color_code: Option<&'static str>,
}

pub struct WordCloud {
    words: Vec<WordCloudObject>,
}

impl WordCloud {
    pub fn new() -> Self {
        return WordCloud { words: Vec::new() };
    }

    pub fn insert(&mut self, text: &str) {
        let trimmed_text = truncate_chars(text.trim(), 64).to_lowercase();

        if let Some(starting_position) =
            self.words.iter().position(|word| word.text == trimmed_text)
        {
            self.words[starting_position].count += 1;

            if let Some(target_pos) = self
                .words
                .iter()
                .position(|word| word.count == self.words[starting_position].count - 1)
            {
                if target_pos < starting_position {
                    let temp = self.words.remove(starting_position);
                    self.words.insert(target_pos, temp);
                }
            }
        } else {
            self.words.push(WordCloudObject {
                text: trimmed_text,
                count: 1usize,
                font_size_rem: 0f32,
                top_rem: 0f32,
                color_code: "#000000",
                previous_top_rem: None,
                previous_font_size_rem: None,
                previous_color_code: None,
            });
        }
    }

    pub fn render<'a>(&'a mut self) -> (&'a [WordCloudObject], f32) {
        let max_count: usize = self
            .words
            .iter()
            .map(|word| word.count)
            .max()
            .unwrap_or(1usize);

        for (i, word) in self.words.iter_mut().enumerate() {
            word.font_size_rem = 0.5f32 + (word.count as f32 / max_count as f32) * 2f32;
            word.previous_color_code = Some(word.color_code);
            word.color_code = WORD_CLOUD_COLORS[i % WORD_CLOUD_COLORS.len()];
        }

        for i in 0..self.words.len() {
            if i == 0 {
                self.words[i].top_rem = 0f32;
            } else {
                self.words[i].top_rem = self.words[i - 1].top_rem
                    + self.words[i - 1].font_size_rem
                    + VERTICAL_WORD_GAP_REM;
            }
        }

        let container_height = self
            .words
            .last()
            .map(|last| last.top_rem + last.font_size_rem)
            .unwrap_or(0f32);

        return (&self.words, container_height);
    }

    pub fn save_previous(&mut self) {
        for word in &mut self.words {
            word.previous_top_rem = Some(word.top_rem);
            word.previous_font_size_rem = Some(word.font_size_rem);
        }
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}
