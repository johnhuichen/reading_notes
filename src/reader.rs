use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use snafu::{Location, ResultExt, Snafu};

use crate::llm::{LLMError, LLM};
use crate::macros::retry;
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

    fn prompt_guide_line() -> String {
        r#"
- Summarize concisely without adding introductory phrases like 'Here is a summary of the ...' or 'In summary:'. Provide only the summary itself without prefatory sentences.
- Maintain a professional tone.
- Do not end by asking a question.
- Strive to be clear and concise.
"#
        .to_string()
    }

    async fn summarize_book(&self, book: &WealthOfNationsBook) -> Result<(), ReaderError> {
        log::info!("{}", book.title);

        let mut chapter_summaries = String::new();
        let mut file = OpenOptions::new().append(true).open(&self.summary_path)?;
        file.write_all(book.title.as_bytes())?;
        file.write_all("\n\n".as_bytes())?;

        for chapter in &book.chapters {
            let summary = self.summarize_chapter(chapter).await?;
            chapter_summaries.push_str(&summary);
        }

        //         let prompt = format!(
        //             r#"You are an expert book reader.
        //
        // Instructions:
        // You are given the summaries of each chapter in a book. Your task is to write summary of the book.
        //
        // Chapter Summaries:
        // {}"
        //
        // Guidelines:
        // {}"#,
        //             chapter_summaries,
        //             Self::prompt_guide_line(),
        //         );
        //
        //         let summary = retry! {
        //             async {
        //                 self.llm.generate_string(&prompt).await
        //             }.await
        //         }
        //         .context(LLMSnafu)?;

        // file.write_all(summary.as_bytes())?;
        // file.write_all("\n\n".as_bytes())?;
        // file.write_all(chapter_summaries.as_bytes())?;

        // log::info!("Summary of the book:");
        // log::info!("{}", summary);

        Ok(())
    }

    async fn summarize_chapter(
        &self,
        chapter: &WealthOfNationsChapter,
    ) -> Result<String, ReaderError> {
        let mut paragraph_summaries = String::new();
        let mut file = OpenOptions::new().append(true).open(&self.summary_path)?;

        for paragraph in &chapter.paragraphs {
            let summary = self.summarize_paragraph(paragraph).await?;
            paragraph_summaries.push_str(&summary);
        }

        let prompt = format!(
            r#"You are an expert book reader.

Instructions:
You are given the content of a chapter. Your task is to write a summary of the chapter.

Content of Chapter:
{}

Guidelines:
- Summarize the chapter in under 300 words and keep everything in one paragraph.
{}"#,
            paragraph_summaries,
            Self::prompt_guide_line(),
        );

        let summary = retry! {
            async {
                self.llm.generate_string(&prompt).await
            }.await
        }
        .context(LLMSnafu)?;

        let result = format!("{}\n\n{}\n\n", chapter.title, summary);

        log::info!("{}", result);
        file.write_all(result.as_bytes())?;

        Ok(result)
    }

    async fn summarize_paragraph(
        &self,
        paragraph: &WealthOfNationsParagraph,
    ) -> Result<String, ReaderError> {
        let prompt = format!(
            r#"You are an expert book.

Instructions:
You are given the content of a paragraph. Your task is to write a summary.

Content of Paragraph:
{}

Guidelines:
- Summarize the paragraph in ONE sentence.
{}"#,
            paragraph.content,
            Self::prompt_guide_line()
        );

        let summary = retry! {
            async {
                self.llm.generate_string(&prompt).await
            }.await
        }
        .context(LLMSnafu)?;

        let result = format!("{}\n\n", summary);
        log::info!("{}", result);

        Ok(result)
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
