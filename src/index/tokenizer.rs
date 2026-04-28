use std::{
    fs::{self, File},
    io::Cursor,
    path::Path,
    sync::Arc,
    time::Duration,
};

use log::warn;
use memmap2::Mmap;
use sudachi::analysis::Tokenize;
use sudachi::analysis::stateless_tokenizer::StatelessTokenizer;
use sudachi::config::Config;
use sudachi::dic::dictionary::JapaneseDictionary;
use sudachi::dic::storage::{Storage, SudachiDicData};
use sudachi::prelude::*;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

const SUDACHI_SYSTEM_DIC_PATH: &str = "sudachi/system.dic";
const SUDACHI_DICTIONARY_URLS: [&str; 2] = [
    "https://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-20260116-full.zip",
    "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict/sudachi-dictionary-20260116-full.zip",
];
const SUDACHI_DICTIONARY_ENTRY: &str = "sudachi-dictionary-20260116/system_full.dic";

#[derive(Clone)]
pub struct SudachiTokenizer {
    dictionary: Arc<JapaneseDictionary>,
}

fn load_sudachi_dictionary(
    config: &Config,
    target_path: &Path,
) -> Result<JapaneseDictionary, BoxError> {
    if let Ok(dictionary) = open_sudachi_dictionary(config, target_path) {
        return Ok(dictionary);
    }

    warn!(
        "Sudachi dictionary not found at {}. Downloading a fresh copy.",
        target_path.display()
    );
    download_sudachi_dictionary(target_path)?;
    open_sudachi_dictionary(config, target_path)
}

fn open_sudachi_dictionary(
    config: &Config,
    target_path: &Path,
) -> Result<JapaneseDictionary, BoxError> {
    let file = File::open(target_path)?;
    let mapping = unsafe { Mmap::map(&file) }?;
    let storage = SudachiDicData::new(Storage::File(mapping));
    Ok(JapaneseDictionary::from_cfg_storage_with_embedded_chardef(
        config, storage,
    )?)
}

fn download_sudachi_dictionary(target_path: &Path) -> Result<(), BoxError> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    for url in SUDACHI_DICTIONARY_URLS {
        if download_dict(&client, url, target_path).is_ok() {
            return Ok(());
        }
    }

    Err(format!(
        "Failed to download Sudachi dictionary to {}. Place a Sudachi dictionary manually at that path and run cargo again.",
        target_path.display()
    )
    .into())
}

fn download_dict(
    client: &reqwest::blocking::Client,
    url: &str,
    target_path: &Path,
) -> Result<(), BoxError> {
    let response = client.get(url).send()?.error_for_status()?;
    let bytes = response.bytes()?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut entry = archive.by_name(SUDACHI_DICTIONARY_ENTRY)?;
    let temp_path = target_path.with_extension("dic.part");

    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    {
        let mut out = fs::File::create(&temp_path)?;
        std::io::copy(&mut entry, &mut out)?;
    }

    if target_path.exists() {
        fs::remove_file(target_path)?;
    }
    fs::rename(&temp_path, target_path)?;

    Ok(())
}

impl SudachiTokenizer {
    pub fn new() -> Result<Self, BoxError> {
        let config = Config::new_embedded()?;
        let dict = Arc::new(load_sudachi_dictionary(
            &config,
            Path::new(SUDACHI_SYSTEM_DIC_PATH),
        )?);
        Ok(Self { dictionary: dict })
    }

    fn new_tokenizer(&self) -> StatelessTokenizer<Arc<JapaneseDictionary>> {
        StatelessTokenizer::new(Arc::clone(&self.dictionary))
    }

    pub fn tokenize(
        &self,
        text: &str,
        mode: Mode,
    ) -> Result<Tokenized, Box<dyn std::error::Error + Send + Sync>> {
        // 文字数じゃなくバイトで切る（UTF-8デコードの全走査を避ける）
        const MAX_CHUNK_BYTES: usize = 16 * 1024; // 例: 16KB（調整してOK）
        const DELIMS: &[u8] = b"\n\r"; // まずは改行だけを境界にすると速い（句読点までやるなら後述）

        let tokenizer = self.new_tokenizer();

        // 短ければそのまま
        if text.len() <= MAX_CHUNK_BYTES {
            let r = tokenizer.tokenize(text, mode, false)?;
            return Ok(Tokenized { result: vec![r] });
        }

        let bytes = text.as_bytes();
        let mut out: Vec<MorphemeList<Arc<JapaneseDictionary>>> = Vec::new();

        let mut start = 0usize;
        while start < bytes.len() {
            let mut end = (start + MAX_CHUNK_BYTES).min(bytes.len());

            // UTF-8境界に合わせる
            while end < bytes.len() && !text.is_char_boundary(end) {
                end -= 1;
            }
            if end <= start {
                // 極端に境界が合わないケースの保険
                end = (start + 1).min(bytes.len());
                while end < bytes.len() && !text.is_char_boundary(end) {
                    end += 1;
                }
            }

            // できれば改行まで後退（軽い delimiter）
            let mut cut = end;
            let search_start = start.max(end.saturating_sub(MAX_CHUNK_BYTES));
            for i in (search_start..end).rev() {
                if DELIMS.contains(&bytes[i]) {
                    cut = i + 1;
                    break;
                }
            }

            let chunk = &text[start..cut];
            if !chunk.trim().is_empty() {
                out.push(tokenizer.tokenize(chunk, mode, false)?);
            }

            start = cut;
        }

        Ok(Tokenized { result: out })
    }

    pub fn mix_doc_tokenizer(
        &self,
        text: &str,
    ) -> Result<(Vec<Box<str>>, u64), Box<dyn std::error::Error + Send + Sync>> {
        let c = self.tokenize(text, Mode::C)?;
        let a = self.tokenize(text, Mode::A)?;
        let mut c_tokens = c.tokens();
        let token_sum = c_tokens.len();
        let a_tokens = a.tokens();
        c_tokens.sort_unstable();
        let c_tokens_sub: Vec<Box<str>> = a_tokens
            .iter()
            .filter(|t| !c_tokens.binary_search(*t).is_ok())
            .cloned()
            .collect();
        let a_speech_tokens = a.speech_tokens();
        let synthetic_tokens: Vec<Box<str>> = c_tokens
            .into_iter()
            .chain(c_tokens_sub.into_iter())
            .chain(a_speech_tokens.into_iter())
            .collect();
        Ok((synthetic_tokens, token_sum as u64))
    }

    pub fn pure_doc_tokenizer(
        &self,
        text: &str,
    ) -> Result<(Vec<Box<str>>, u64), Box<dyn std::error::Error + Send + Sync>> {
        let c = self.tokenize(text, Mode::C)?;
        let token_sum = c.tokens().len();
        Ok((c.tokens(), token_sum as u64))
    }

    pub fn mix_query_tokenizer(
        &self,
        text: &str,
    ) -> Result<Vec<Box<str>>, Box<dyn std::error::Error + Send + Sync>> {
        let c = self.tokenize(text, Mode::C)?;
        let a = self.tokenize(text, Mode::A)?;
        let mut c_tokens = c.tokens();
        let a_tokens = a.tokens();
        c_tokens.sort();
        let c_tokens_sub: Vec<Box<str>> = a_tokens
            .iter()
            .filter(|t| !c_tokens.binary_search(*t).is_ok())
            .cloned()
            .collect();
        let a_2gram_tokens: Vec<Box<str>> = a_tokens
            .windows(2)
            .map(|w| format!("{}{}", w[0], w[1]).into_boxed_str())
            .collect();
        let a_speech_tokens = a.speech_tokens();
        let synthetic_tokens: Vec<Box<str>> = c_tokens
            .into_iter()
            .chain(c_tokens_sub.into_iter())
            .chain(a_speech_tokens.into_iter())
            .chain(a_2gram_tokens.into_iter())
            .collect();
        Ok(synthetic_tokens)
    }
}

pub struct Tokenized {
    result: Vec<MorphemeList<Arc<JapaneseDictionary>>>,
}

impl Tokenized {
    // pub fn normalized_tokens(&self) -> Vec<Box<str>> {
    //     self.result
    //         .iter().flat_map(|m| {
    //             m.iter()
    //                 .map(|s| s.normalized_form().trim_matches(&[' ', '　']).to_string().into_boxed_str())
    //                 .filter(|s| !s.is_empty())
    //                 .collect::<Vec<Box<str>>>()
    //         }).collect::<Vec<Box<str>>>()
    // }

    pub fn tokens(&self) -> Vec<Box<str>> {
        self.result
            .iter()
            .flat_map(|m| {
                m.iter()
                    .map(|s| {
                        s.surface()
                            .trim_matches(&[' ', '　'])
                            .to_string()
                            .into_boxed_str()
                    })
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<Box<str>>>()
            })
            .collect::<Vec<Box<str>>>()
    }

    pub fn speech_tokens(&self) -> Vec<Box<str>> {
        self.result
            .iter()
            .flat_map(|m| {
                m.iter()
                    .map(|s| {
                        s.reading_form()
                            .replace("キゴウ", "")
                            .trim()
                            .to_string()
                            .into_boxed_str()
                    })
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<Box<str>>>()
            })
            .collect::<Vec<Box<str>>>()
    }
}
