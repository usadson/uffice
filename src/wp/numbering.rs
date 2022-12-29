// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{
    collections::HashMap,
    rc::Rc,
    cell::RefCell
};

use roxmltree as xml;

use crate::{WORD_PROCESSING_XML_NAMESPACE, text_settings::TextSettings};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum NumberingFormat {
    Aiueo,
    AiueoFullWidth,
    ArabicAbjad,
    ArabicAlpha,
    BahtText,
    Bullet,
    CardinalText,
    Chicago,
    ChineseCounting,
    ChineseCountingThousand,
    ChineseLegalSimplified,
    Chosung,
    Custom(String),
    Decimal,
    DecimalEnclosedCircle,
    DecimalEnclosedCircleChinese,
    DecimalEnclosedFullstop,
    DecimalEnclosedParen,
    DecimalFullWidth,
    DecimalHalfWidth,
    DecimalZero,
    DollarText,
    Ganada,
    Hebrew1,
    Hebrew2,
    Hex,
    HindiConsonants,
    HindiCounting,
    HindiNumbers,
    HindiVowels,
    IdeographDigital,
    IdeographEnclosedCircle,
    IdeographLegalTraditional,
    IdeographTraditional,
    IdeographZodiac,
    IdeographZodiacTraditional,
    Iroha,
    IrohaFullWidth,
    JapaneseCounting,
    JapaneseDigitalTenThousand,
    JapaneseLegal,
    KoreanCounting,
    KoreanDigital,
    KoreanDigital2,
    KoreanLegal,
    LowerLetter,
    LowerRoman,
    None,
    NumberInDash,
    Ordinal,
    OrdinalText,
    RussianLower,
    RussianUpper,
    TaiwaneseCounting,
    TaiwaneseCountingThousand,
    TaiwaneseDigital,
    ThaiCounting,
    ThaiLetters,
    ThaiNumbers,
    UpperLetter,
    UpperRoman,
    VietnameseCounting,
}

impl NumberingFormat {

    pub fn parse(value: &str) -> Option<NumberingFormat> {
        match value {
            "aiueo" => Some(NumberingFormat::Aiueo),
            "aiueoFullWidth" => Some(NumberingFormat::AiueoFullWidth),
            "arabicAbjad" => Some(NumberingFormat::ArabicAbjad),
            "arabicAlpha" => Some(NumberingFormat::ArabicAlpha),
            "bahtText" => Some(NumberingFormat::BahtText),
            "bullet" => Some(NumberingFormat::Bullet),
            "cardinalText" => Some(NumberingFormat::CardinalText),
            "chicago" => Some(NumberingFormat::Chicago),
            "chineseCounting" => Some(NumberingFormat::ChineseCounting),
            "chineseCountingThousand" => Some(NumberingFormat::ChineseCountingThousand),
            "chineseLegalSimplified" => Some(NumberingFormat::ChineseLegalSimplified),
            "chosung" => Some(NumberingFormat::Chosung),
            "decimal" => Some(NumberingFormat::Decimal),
            "decimalEnclosedCircle" => Some(NumberingFormat::DecimalEnclosedCircle),
            "decimalEnclosedCircleChinese" => Some(NumberingFormat::DecimalEnclosedCircleChinese),
            "decimalEnclosedFullstop" => Some(NumberingFormat::DecimalEnclosedFullstop),
            "decimalEnclosedParen" => Some(NumberingFormat::DecimalEnclosedParen),
            "decimalFullWidth" => Some(NumberingFormat::DecimalFullWidth),
            "decimalHalfWidth" => Some(NumberingFormat::DecimalHalfWidth),
            "decimalZero" => Some(NumberingFormat::DecimalZero),
            "dollarText" => Some(NumberingFormat::DollarText),
            "ganada" => Some(NumberingFormat::Ganada),
            "hebrew1" => Some(NumberingFormat::Hebrew1),
            "hebrew2" => Some(NumberingFormat::Hebrew2),
            "hex" => Some(NumberingFormat::Hex),
            "hindiConsonants" => Some(NumberingFormat::HindiConsonants),
            "hindiCounting" => Some(NumberingFormat::HindiCounting),
            "hindiNumbers" => Some(NumberingFormat::HindiNumbers),
            "hindiVowels" => Some(NumberingFormat::HindiVowels),
            "ideographDigital" => Some(NumberingFormat::IdeographDigital),
            "ideographEnclosedCircle" => Some(NumberingFormat::IdeographEnclosedCircle),
            "ideographLegalTraditional" => Some(NumberingFormat::IdeographLegalTraditional),
            "ideographTraditional" => Some(NumberingFormat::IdeographTraditional),
            "ideographZodiac" => Some(NumberingFormat::IdeographZodiac),
            "ideographZodiacTraditional" => Some(NumberingFormat::IdeographZodiacTraditional),
            "iroha" => Some(NumberingFormat::Iroha),
            "irohaFullWidth" => Some(NumberingFormat::IrohaFullWidth),
            "japaneseCounting" => Some(NumberingFormat::JapaneseCounting),
            "japaneseDigitalTenThousand" => Some(NumberingFormat::JapaneseDigitalTenThousand),
            "japaneseLegal" => Some(NumberingFormat::JapaneseLegal),
            "koreanCounting" => Some(NumberingFormat::KoreanCounting),
            "koreanDigital" => Some(NumberingFormat::KoreanDigital),
            "koreanDigital2" => Some(NumberingFormat::KoreanDigital2),
            "koreanLegal" => Some(NumberingFormat::KoreanLegal),
            "lowerLetter" => Some(NumberingFormat::LowerLetter),
            "lowerRoman" => Some(NumberingFormat::LowerRoman),
            "none" => Some(NumberingFormat::None),
            "numberInDash" => Some(NumberingFormat::NumberInDash),
            "ordinal" => Some(NumberingFormat::Ordinal),
            "ordinalText" => Some(NumberingFormat::OrdinalText),
            "russianLower" => Some(NumberingFormat::RussianLower),
            "russianUpper" => Some(NumberingFormat::RussianUpper),
            "taiwaneseCounting" => Some(NumberingFormat::TaiwaneseCounting),
            "taiwaneseCountingThousand" => Some(NumberingFormat::TaiwaneseCountingThousand),
            "taiwaneseDigital" => Some(NumberingFormat::TaiwaneseDigital),
            "thaiCounting" => Some(NumberingFormat::ThaiCounting),
            "thaiLetters" => Some(NumberingFormat::ThaiLetters),
            "thaiNumbers" => Some(NumberingFormat::ThaiNumbers),
            "upperLetter" => Some(NumberingFormat::UpperLetter),
            "upperRoman" => Some(NumberingFormat::UpperRoman),
            "vietnameseCounting" => Some(NumberingFormat::VietnameseCounting),
            _ => None
        }
    }

}

#[derive(Clone, Debug)]
pub struct NumberingLevelDefinition {
    display_all_levels_using_arabic_numerals: bool,
    format: NumberingFormat,
    starting_value: i32,
    text: String,
    pub text_settings: TextSettings,

    pub current_value: Option<i32>,
}

impl NumberingLevelDefinition {
    pub fn load_xml(node: &xml::Node) -> Self {
        let mut definition = Self {
            display_all_levels_using_arabic_numerals: false,
            format: NumberingFormat::Decimal,
            starting_value: 0,
            text: String::new(),
            text_settings: TextSettings::new(),
            current_value: None,
        };

        for child in node.children() {
            match child.tag_name().name() {

                // 17.9.4 isLgl (Display All Levels Using Arabic Numerals)
                //
                // This element specifies whether or not all levels displayed
                // for a given numbering level's text shall be displayed using
                // the decimal number format, regardless of the actual number
                // format of that level in the list.
                //
                // Note: This numbering style is often referred to as the legal
                //       numbering style.
                "isLgl" => {
                    definition.display_all_levels_using_arabic_numerals = true;
                }

                // 17.9.7 lvlJc (Justification)
                //
                // This element specifies the type of justification used on a
                // numbering level's text within a given numbering level.
                "lvlJc" => {
                    // TODO
                }

                // 17.9.11 lvlText (Numbering Level Text)
                //
                // This element specifies the textual content which shall be
                // displayed when displaying a paragraph with the given
                // numbering level.
                "lvlTxt" => {
                    definition.text = String::from(
                        child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                                .expect("No w:val given for a <w:lvlTxt>!")
                    );
                }

                // 17.9.17 numFmt (Numbering Format)
                "numFmt" => {
                    let val = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                        .expect("No w:val given for a <w:numFmt>!");

                    if val == "custom" {
                        definition.format = NumberingFormat::Custom(String::from(
                            child.attribute((WORD_PROCESSING_XML_NAMESPACE, "format"))
                                .expect("No w:format attribute for a <w:numFmt w:val=\"custom\">!")
                        ));
                    } else {
                        definition.format = NumberingFormat::parse(val).unwrap();
                    }
                }

                "pPr" => definition.parse_number_level_associated_paragraph_properties(&child),

                // 17.9.25 start (Starting Value)
                "start" => {
                    definition.starting_value = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                        .expect("No w:val given for a <w:abstractNumId>!").parse().unwrap();
                }

                _ => ()
            }
        }

        definition
    }

    fn parse_number_level_associated_paragraph_properties(&mut self, node: &xml::Node) {
        for child in node.children() {
            match child.tag_name().name() {
                "ind" => {
                    if let Some(value) = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "left")) {
                        self.text_settings.indentation_left = Some(value.parse().unwrap());
                    }
                    if let Some(value) = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "hanging")) {
                        self.text_settings.indentation_hanging = Some(value.parse().unwrap());
                    }
                }

                _ => ()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AbstractNumberingDefinition {
    pub levels: HashMap<i32, Rc<RefCell<NumberingLevelDefinition>>>,
}

#[derive(Clone, Debug)]
pub struct NumberingDefinitionInstance {
    pub abstract_numbering_definition: Option<Rc<RefCell<AbstractNumberingDefinition>>>,
}

#[derive(Debug)]
pub struct NumberingManager {
    pub abstract_numbering_definitions: HashMap<i32, Rc<RefCell<AbstractNumberingDefinition>>>,
    pub numbering_definition_instances: HashMap<i32, Rc<RefCell<NumberingDefinitionInstance>>>,

    pub values: Vec<i32>,
}

impl NumberingManager {
    pub fn from_xml(doc: &xml::Document) -> Self {
        let mut manager = Self {
            abstract_numbering_definitions: HashMap::new(),
            numbering_definition_instances: HashMap::new(),
            values: Vec::new(),
        };

        for node in doc.root_element().children() {
            match node.tag_name().name() {
                // 17.9.1 abstractNum (Abstract Numbering Definition)
                "abstractNum" => manager.parse_abstract_numbering_definition(&node),

                // 17.9.15 num (Numbering Definition Instance)
                "num" => manager.parse_numbering_definition_instance(&node),

                _ => ()
            }
        }

        manager
    }

    pub fn find_definition_instance(&self, id: i32) -> Option<Rc<RefCell<NumberingDefinitionInstance>>> {
        self.numbering_definition_instances.get(&id).cloned()
    }

    fn parse_abstract_numbering_definition(&mut self, node: &xml::Node) {
        let abstract_num_id: i32 = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "abstractNumId"))
                .expect("No w:abstractNumId given for a <w:abstractNum>!").parse().unwrap();

        if self.numbering_definition_instances.contains_key(&abstract_num_id) {
            panic!("Duplicate <w:abstractNum> for abstractNumId: {}", abstract_num_id);
        }

        let mut definition = AbstractNumberingDefinition{
            levels: HashMap::new()
        };

        for child in node.children() {
            match child.tag_name().name() {

                // 17.9.6 lvl (Numbering Level Definition)
                "lvl" => {
                    let id: i32 = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "ilvl"))
                        .expect("No w:ilvl given for a <w:lvl>!").parse().unwrap();

                    definition.levels.insert(id, Rc::new(RefCell::new(NumberingLevelDefinition::load_xml(&child))));
                }

                _ => ()
            }
        }

        self.abstract_numbering_definitions.insert(abstract_num_id, Rc::new(RefCell::new(definition)));
    }

    fn parse_numbering_definition_instance(&mut self, node: &xml::Node) {
        let id: i32 = node.attribute((WORD_PROCESSING_XML_NAMESPACE, "numId"))
                .expect("No w:numId given for a <w:num>!").parse().unwrap();

        if self.numbering_definition_instances.contains_key(&id) {
            panic!("Duplicate <w:num> for id: {}", id);
        }

        let mut instance = NumberingDefinitionInstance{
            abstract_numbering_definition: None
        };

        for child in node.children() {
            match child.tag_name().name() {
                // 17.9.2 abstractNumId (Abstract Numbering Definition Reference)
                "abstractNumId" => {
                    let abstract_num_id: i32 = child.attribute((WORD_PROCESSING_XML_NAMESPACE, "val"))
                        .expect("No w:val given for a <w:abstractNumId>!").parse().unwrap();

                    instance.abstract_numbering_definition = Some(
                        self.abstract_numbering_definitions.get(&abstract_num_id)
                            .expect(&format!("No abstract numbering definition found for <w:abstractNumId> reference: {}",
                                    abstract_num_id))
                            .clone()
                    );
                }

                _ => ()
            }
        }

        self.numbering_definition_instances.insert(id, Rc::new(RefCell::new(instance)));
    }
}
