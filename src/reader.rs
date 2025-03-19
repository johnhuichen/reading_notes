use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{Location, ResultExt, Snafu};

use crate::llm::{LLMError, LLM};
use crate::parser::{
    WealthOfNations, WealthOfNationsBook, WealthOfNationsChapter, WealthOfNationsError,
    WealthOfNationsParagraph,
};

pub trait Reader {
    async fn summarize(&self) -> Result<(), ReaderError>;
}

#[derive(Debug, Snafu)]
pub enum ReaderError {
    #[snafu(display("IO error at {loc}"))]
    #[snafu(context(false))]
    IO {
        source: io::Error,
        #[snafu(implicit)]
        loc: Location,
    },

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
    summary_dir: String,
    summary_path: PathBuf,
}

impl WealthOfNationsReader {
    pub fn new() -> Self {
        let llm = LLM::new();
        let summary_dir = "./summary".to_string();
        let summary_file = "wealth_of_nations".to_string();
        let summary_path = Path::new(&summary_dir).join(&summary_file);

        WealthOfNationsReader {
            llm,
            summary_dir,
            summary_path,
        }
    }

    async fn summarize_book(&self, book: &WealthOfNationsBook) -> Result<(), ReaderError> {
        log::info!("{}", book.title);

        let mut chapter_summaries = String::new();
        let mut file = OpenOptions::new().append(true).open(&self.summary_path)?;
        file.write_all(book.title.as_bytes())?;
        file.write_all("\n\n".as_bytes())?;

        for chapter in &book.chapters {
            let summary = self.summarize_chapter(&chapter_summaries, chapter).await?;
            chapter_summaries.push_str(&summary);
            chapter_summaries.push_str("\n\n");
        }

        let prompt = format!(
            r#"You are an expert book reader summarizing a book. You have summarized each chapter in this book.

Below are the chapter summaries of the book. Use them to write a clear, concise, and well-structured summary of the book.

### Guidelines:
- Identify the book’s main themes, key arguments, and overall message.
- Maintain a professional and informative tone.

### Chapter Summaries:
{}

### Book Summary:
Write a structured summary of the book. Start with a direct statement about its main idea, then summarize the key points, and conclude with its overall significance or takeaway."#,
            chapter_summaries
        );

        let notes: BookNotes = self.llm.generate(&prompt).await.context(LLMSnafu)?;

        file.write_all(notes.summary.as_bytes())?;
        file.write_all("\n\n".as_bytes())?;
        file.write_all(chapter_summaries.as_bytes())?;

        log::info!("Summary of the book:");
        log::info!("{}", notes.summary);

        Ok(())
    }

    async fn summarize_chapter(
        &self,
        previous_summaries: &str,
        chapter: &WealthOfNationsChapter,
    ) -> Result<String, ReaderError> {
        let mut paragraph_summaries = String::new();

        for paragraph in &chapter.paragraphs {
            let summary = self
                .summarize_paragraph(&paragraph_summaries, paragraph)
                .await?;
            paragraph_summaries.push_str(&summary);
            paragraph_summaries.push_str("\n\n");
        }

        let prompt = format!(
            r#"You are an expert book reader summarizing a chapter. You have summarized each paragraph in this chapter, and you may also have summaries of previous chapters for context.

Below are the summaries of previous chapters, followed by the paragraph summaries of the current chapter. Use them to write a detailed, structured, and insightful summary of the current chapter.

### Previous Chapter Summaries:
{}

### Paragraph Summaries for Current Chapter:
{}

### Chapter Summary:
Write a detailed but concise summary of the current chapter. The summary should be comprehensive, covering the key points, examples, and arguments, but avoid unnecessary repetition or overly broad statements.

- Start with a clear and direct statement of the main argument. What is the core idea or thesis?
- Provide detailed explanations for key concepts.
- Analyze the cause-and-effect relationships presented in the chapter.
- Include any significant counterarguments or critiques mentioned in the chapter, as well as the author's response to them.
- Conclude by explaining the overall significance of this chapter in the context of the book.

Ensure that the summary is under 1000 words."#,
            previous_summaries, paragraph_summaries
        );

        let notes: ChapterNotes = self.llm.generate(&prompt).await.context(LLMSnafu)?;

        let result = format!("{}\n\n{}", chapter.title, notes.summary);

        log::info!("{}", result);

        Ok(result)
    }

    async fn summarize_paragraph(
        &self,
        previous_summaries: &str,
        paragraph: &WealthOfNationsParagraph,
    ) -> Result<String, ReaderError> {
        let prompt = format!(
            r#"You are an expert book reader summarizing a chapter, paragraph by paragraph. Your goal is to write a clear, concise, and professional summary of the current paragraph in one to three sentences.

- Identify the key idea or main argument presented in the paragraph.
- Focus on specific concepts, examples, or details provided in the paragraph. Avoid vague language or generalities.
- Maintain a structured and objective tone. Do not start with phrases like "The paragraph explains" as they are redundant.
- Summarize the paragraph's contribution to the chapter's overall message, without adding unnecessary repetition.

### Previous Summaries:
{}

### Current Paragraph:
{}

### Summary:"#,
            previous_summaries, paragraph.content
        );

        let notes: ParagraphNotes = self.llm.generate(&prompt).await.context(LLMSnafu)?;

        log::debug!("{}", notes.summary);

        Ok(notes.summary)
    }
}

impl Reader for WealthOfNationsReader {
    async fn summarize(&self) -> Result<(), ReaderError> {
        fs::create_dir_all(&self.summary_dir)?;
        File::create(&self.summary_path)?;

        let won = WealthOfNations::new().context(WealthOfNationsSnafu)?;

        for book in won.books {
            self.summarize_book(&book).await?;
        }

        Ok(())
    }
}
