// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{
    collections::HashMap,
    rc::Rc,
    cell::RefCell
};

use roxmltree as xml;

use crate::{WORD_PROCESSING_XML_NAMESPACE, text_settings::TextSettings, unicode::alphabet::Alphabet};

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

    /// Specifies that the sequence shall consist of decimal numbering.
    ///
    /// To determine the text that is displayed for any value, this sequence
    /// specifies a set of characters that represent positions 1–9 and then
    /// those same characters are combined with each other and 0 (represents the
    /// number zero) to construct the remaining values.
    ///
    /// The set of characters used by this numbering format for values 0–9 is
    /// U+0030–U+0039, respectively.
    ///
    /// Continue the sequence by using the following steps:
    ///     1.  Increment the rightmost position.
    ///     2.  Every time the end of the set is reached, for a given position,
    ///         increment the position to the immediate left (if there is no
    ///         position to the immediate left, create a new position and start
    ///         the sequence of the new position at 1) and reset the current
    ///         position to 0.
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

    /// Specifies that the sequence shall consist of lowercase roman numerals.
    ///
    /// This system uses a set of characters to represent the numbers 1, 5, 10,
    /// 50, 100, 500, and 1000 and then those are combined with each other to
    /// construct the remaining values.
    ///
    /// The set of characters used by this numbering format is U+0069, U+0076,
    /// U+0078, U+006C, U+0063, U+0064, U+006D, respectively.
    ///
    /// To construct a number that is outside the set, you work from largest
    /// groups to smallest following these steps:
    /// 1.  Create as many groups as possible that contain one thousand in each
    ///     group.
    ///     *   The symbol representing one thousand (the power of ten
    ///         represented by that position): m is repeated for the number of
    ///         groups formed.
    ///         If no groups are formed, do not write any symbol.
    /// 2.  Repeat this for groups of nine hundred (cm), five-hundred (d),
    ///     four-hundred (cd), one-hundred (c), ninety (xc), fifty (l), forty
    ///     (xl), ten (x), nine (ix), five (v), four (iv) and finally one (i)
    ///     using the corresponding symbol to indicate the groups (so
    ///     four-hundred fifty would be cdl and forty-five would be xlv).
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

    /// Specifies that the sequence shall consist of one or more occurrences of
    /// a single letter of the Latin alphabet in upper case, from the set listed
    /// below.
    ///
    /// This system uses a set of characters to represent the numbers 1 to the
    /// length of the language of the alphabet and then those same characters
    /// are combined to construct the remaining values.
    ///
    /// The characters used by this numbering format is determined by using the
    /// language of the lang element (§17.3.2.20). Specifically:
    /// *   When the script in use is derived from the Latin alphabet (A–Z),
    ///     that alphabet is used.
    ///     [Example: For Norwegian (Nyorsk), the following Unicode characters
    ///     are used by this numbering format: U+0041–U+005A, U+00C6, U+00D8,
    ///     U+00C5. end example]
    /// *   When the language in use is based on any other system, the
    ///     characters U+0041–U+005A are used.
    ///
    /// For values greater than the size of the set, the number is constructed
    /// by following these steps:
    /// 1.  Repeatedly subtract the size of the set from the value until the
    ///     result is equal to or less than the size of the set.
    /// 2.  The result value determines which character to use, and the same
    ///     character is written once and then repeated for each time the size
    ///     of the set was subtracted from the original value.
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

    pub fn format(&self, value: i32) -> String {
        match self.format {
            NumberingFormat::Decimal => format!("{}", value),
            NumberingFormat::LowerRoman => {
                // TODO actually follow algorithm ^_^
                match value {
                    1 => 'i'.to_string(),
                    2 => String::from("ii"),
                    3 => String::from("iii"),
                    4 => String::from("iv"),
                    5 => 'v'.to_string(),
                    6 => String::from("vi"),
                    7 => String::from("vii"),
                    8 => String::from("viii"),
                    9 => String::from("ix"),
                    10 => 'x'.to_string(),
                    _ => panic!("TODO support higher values")
                }
            }
            NumberingFormat::None => String::new(),
            NumberingFormat::UpperLetter => {
                // TODO use the alphabet corresponding to the <w:lang>
                // TODO jump from Z to AA, AB,... and AZ to BA, etc.
                assert!(value > 0);
                assert!(value <= 26, "TODO support higher values");
                crate::unicode::alphabet::Latin::nth(value as usize - 1).to_string()
            }
            _ => {
                println!("[Numbering] Unsupported numbering format: {:?}", self.format);
                String::from("UNSUPPORTED FORMAT")
            }
        }
    }

    pub fn current_value(&self) -> i32 {
        self.current_value.unwrap_or(self.starting_value)
    }

    pub fn next_value(&mut self) -> i32 {
        match self.current_value {
            Some(value) => {
                self.current_value = Some(value + 1);
                value + 1
            }
            None => {
                self.current_value = Some(self.starting_value);
                self.starting_value
            }
        }
    }

    fn parse_number_level_associated_paragraph_properties(&mut self, node: &xml::Node) {
        for child in node.children() {
            match child.tag_name().name() {
                "ind" => self.text_settings.parse_element_ind(node),

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
    pub fn new() -> Self {
        Self {
            abstract_numbering_definitions: HashMap::new(),
            numbering_definition_instances: HashMap::new(),

            values: Vec::new(),
        }
    }

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
