use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum ReaderError {
    #[snafu(display("IO error at {loc}"))]
    #[snafu(context(false))]
    IO {
        source: io::Error,
        #[snafu(implicit)]
        loc: Location,
    },
}

pub trait Reader<L> {
    fn read(&self) -> Result<L, ReaderError>;
}

pub struct WealthOfNationsReader {
    file_path: PathBuf,
}

#[derive(Debug)]
pub struct WealthOfNations {
    books: Vec<WealthOfNationsBook>,
}

impl WealthOfNations {
    fn add_book(&mut self, book: WealthOfNationsBook) {
        self.books.push(book);
    }

    fn add_chapter(&mut self, chapter: WealthOfNationsChapter) {
        self.books.last_mut().unwrap().add_chapter(chapter);
    }

    fn add_paragraph(&mut self, paragraph: WealthOfNationsParagraph) {
        self.books.last_mut().unwrap().add_paragraph(paragraph);
    }
}

#[derive(Debug)]
pub struct WealthOfNationsBook {
    title: String,
    chapters: Vec<WealthOfNationsChapter>,
}

impl WealthOfNationsBook {
    fn add_chapter(&mut self, chapter: WealthOfNationsChapter) {
        self.chapters.push(chapter);
    }

    fn add_paragraph(&mut self, paragraph: WealthOfNationsParagraph) {
        self.chapters.last_mut().unwrap().add_paragraph(paragraph);
    }
}

#[derive(Debug)]
pub struct WealthOfNationsChapter {
    title: String,
    paragraphs: Vec<WealthOfNationsParagraph>,
}

impl WealthOfNationsChapter {
    fn add_paragraph(&mut self, paragraph: WealthOfNationsParagraph) {
        self.paragraphs.push(paragraph);
    }
}

#[derive(Debug)]
pub struct WealthOfNationsParagraph {
    content: String,
}

impl WealthOfNationsReader {
    pub fn new() -> Self {
        // let file_path = Path::new("./data").join("wealth_of_nations");
        let file_path = Path::new("./data").join("wealth_of_nations_book1");

        WealthOfNationsReader { file_path }
    }
}

impl Reader<WealthOfNations> for WealthOfNationsReader {
    fn read(&self) -> Result<WealthOfNations, ReaderError> {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        let mut result = WealthOfNations { books: Vec::new() };

        while reader.read_line(&mut buffer).is_ok_and(|x| x > 0) {
            if buffer.starts_with("BOOK") {
                buffer = buffer.trim_end().to_string();
                reader.read_line(&mut buffer)?;
                let book = WealthOfNationsBook {
                    title: buffer.to_string(),
                    chapters: Vec::new(),
                };
                result.add_book(book);
            } else if buffer.starts_with("CHAPTER") {
                buffer = buffer.trim_end().to_string();
                reader.read_line(&mut buffer)?;
                let chapter = WealthOfNationsChapter {
                    title: buffer.to_string(),
                    paragraphs: Vec::new(),
                };
                result.add_chapter(chapter);
            } else if buffer.len() >= 70 {
                let paragraph = WealthOfNationsParagraph {
                    content: buffer.trim().to_string(),
                };
                result.add_paragraph(paragraph);
            }
            buffer.clear();
        }

        Ok(result)
    }
}
