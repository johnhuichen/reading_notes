use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::llm::{LLMError, LLM};
use crate::parser::{
    WealthOfNations, WealthOfNationsBook, WealthOfNationsChapter, WealthOfNationsError,
    WealthOfNationsParagraph,
};

pub trait Reader {
    async fn summarize(&self) -> Result<String, ReaderError>;
}

#[derive(Debug, Snafu)]
pub enum ReaderError {
    #[snafu(display("Failed to run LLM"))]
    #[allow(clippy::upper_case_acronyms)]
    LLM { source: LLMError },

    #[snafu(display("Failed to parse Wealth of Nations"))]
    WealthOfNations { source: WealthOfNationsError },
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ParagraphNotes {
    summary: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ChapterNotes {
    summary: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct BookNotes {
    summary: String,
}

pub struct WealthOfNationsReader {
    llm: LLM,
}

impl WealthOfNationsReader {
    pub fn new() -> Self {
        let llm = LLM::new();
        WealthOfNationsReader { llm }
    }

    async fn summarize_book(&self, book: &WealthOfNationsBook) -> Result<String, ReaderError> {
        let mut chapter_summaries = String::new();

        for chapter in &book.chapters {
            let notes = self.summarize_chapter(&chapter_summaries, chapter).await?;
            chapter_summaries.push_str(&notes);
            chapter_summaries.push_str("\n\n");
        }

        Ok(chapter_summaries)
    }

    async fn summarize_chapter(
        &self,
        previous_summaries: &str,
        chapter: &WealthOfNationsChapter,
    ) -> Result<String, ReaderError> {
        let mut paragraph_summaries = String::new();

        for paragraph in &chapter.paragraphs {
            let notes = self
                .summarize_paragraph(&paragraph_summaries, paragraph)
                .await?;
            paragraph_summaries.push_str(&notes);
            paragraph_summaries.push_str("\n\n");
        }

        let prompt = format!(
            r#"You are an expert book reader summarizing a chapter. You have summarized each paragraph in this chapter, and you may also have summaries of previous chapters for context.

Below are the summaries of previous chapters, followed by the paragraph summaries of the current chapter. Use them to write a clear, concise, and well-structured summary of the current chapter.

### Guidelines:
- Identify the chapter’s main themes, key arguments, and overall message.
- Maintain a professional and informative tone.
- Avoid formulaic or generic openings such as "The current chapter builds upon," "This chapter discusses,". Instead, start with a direct statement about the chapter’s main idea.

### Previous Chapter Summaries:
{}

### Paragraph Summaries for Current Chapter:
{}

### Chapter Summary:
Write a structured summary of the current chapter. Start with a direct statement about its main idea, then summarize the key points, and conclude with its overall significance or takeaway."#,
            previous_summaries, paragraph_summaries
        );

        let notes: ChapterNotes = self.llm.generate(&prompt).await.context(LLMSnafu)?;

        Ok(notes.summary)
    }

    async fn summarize_paragraph(
        &self,
        previous_summaries: &str,
        paragraph: &WealthOfNationsParagraph,
    ) -> Result<String, ReaderError> {
        let prompt = format!(
            r#"You are an expert book reader summarizing a chapter, paragraph by paragraph. Your goal is to write a clear, concise, and professional summary of the current paragraph in one to three sentences.

Avoid phrases like "The paragraph explains" or "So," at the beginning. Instead, summarize the key idea naturally and objectively, maintaining a professional and structured tone.

Previous Summaries:
{}

Current Paragraph:
{}

Summary:"#,
            previous_summaries, paragraph.content
        );

        let notes: ParagraphNotes = self.llm.generate(&prompt).await.context(LLMSnafu)?;

        Ok(notes.summary)
    }
}

impl Reader for WealthOfNationsReader {
    async fn summarize(&self) -> Result<String, ReaderError> {
        let won = WealthOfNations::new().context(WealthOfNationsSnafu)?;
        let mut result = String::new();

        for book in won.books {
            let notes = self.summarize_book(&book).await?;
            result.push_str(&notes);
        }

        Ok(result)
    }
}
