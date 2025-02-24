use std::path::PathBuf;

use ocrs::{ImageSource, OcrEngine, OcrEngineParams};

use crate::{
    constants::APP_ID,
    pages::gallery::page::{PATH_TO_TEXT_DETECTION_MODEL, PATH_TO_TEXT_RECOGNITION_MODEL},
};

pub fn run_ocr(image_path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let idirfein_data_dir = dirs::data_dir()
        .expect("Can't find data dir")
        .as_path()
        .join(APP_ID);
    let detection_model =
        rten::Model::load_file(idirfein_data_dir.join(PATH_TO_TEXT_DETECTION_MODEL))?;
    let recognition_model =
        rten::Model::load_file(idirfein_data_dir.join(PATH_TO_TEXT_RECOGNITION_MODEL))?;

    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;

    let img = image::open(image_path).map(|image| image.into_rgb8())?;

    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;

    let word_rects = engine.detect_words(&ocr_input)?;

    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);

    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    Ok(line_texts
        .iter()
        .flatten()
        .filter(|l| l.to_string().len() > 1)
        .fold(String::new(), |acc, item| acc + " " + &item.to_string()))
}
