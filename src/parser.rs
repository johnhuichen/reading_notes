use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum WealthOfNationsError {
    #[snafu(display("IO error at {loc}"))]
    #[snafu(context(false))]
    IO {
        source: io::Error,
        #[snafu(implicit)]
        loc: Location,
    },
}

#[derive(Debug)]
pub struct WealthOfNations {
    pub books: Vec<WealthOfNationsBook>,
}

impl WealthOfNations {
    pub fn new() -> Result<Self, WealthOfNationsError> {
        // let file_path = Path::new("./data").join("wealth_of_nations");
        let file_path = Path::new("./data").join("wealth_of_nations_book1");
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        let mut result = WealthOfNations { books: Vec::new() };

        while reader.read_line(&mut buffer).is_ok_and(|x| x > 0) {
            if buffer.starts_with("BOOK") {
                buffer = buffer.trim().to_string();
                reader.read_line(&mut buffer)?;
                let book = WealthOfNationsBook {
                    title: buffer.trim().to_string(),
                    chapters: Vec::new(),
                };
                result.add_book(book);
            } else if buffer.starts_with("CHAPTER") {
                buffer = buffer.trim().to_string();
                reader.read_line(&mut buffer)?;
                let chapter = WealthOfNationsChapter {
                    title: buffer.trim().to_string(),
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
    pub title: String,
    pub chapters: Vec<WealthOfNationsChapter>,
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
    pub title: String,
    pub paragraphs: Vec<WealthOfNationsParagraph>,
}

impl WealthOfNationsChapter {
    fn add_paragraph(&mut self, paragraph: WealthOfNationsParagraph) {
        self.paragraphs.push(paragraph);
    }
}

#[derive(Debug)]
pub struct WealthOfNationsParagraph {
    pub content: String,
}
